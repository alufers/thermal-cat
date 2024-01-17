use std::{fmt, sync::Arc, thread};

use eframe::epaint::mutex::Mutex;

use rusb::{Context, Device, HotplugBuilder, UsbContext};

#[derive(Copy, Clone)]
pub enum HotplugEvent {
    DeviceArrived { vendor_id: u16, product_id: u16 },
    DeviceLeft { vendor_id: u16, product_id: u16 },
}

impl fmt::Display for HotplugEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HotplugEvent::DeviceArrived {
                vendor_id,
                product_id,
            } => {
                // print as hex
                write!(
                    f,
                    "DeviceArrived {{ vendor_id: {:#06x}, product_id:  {:#06x} }}",
                    vendor_id, product_id
                )
            }
            HotplugEvent::DeviceLeft {
                vendor_id,
                product_id,
            } => {
                write!(
                    f,
                    "DeviceLeft {{ vendor_id: {:#06x}, product_id: {:#06x} }}",
                    vendor_id, product_id
                )
            }
        }
    }
}

type HotplugtEventCallback = dyn Fn(HotplugEvent) + Send;

pub struct HotplugDetector {
    pub receiver: std::sync::mpsc::Receiver<HotplugEvent>,
    callback: Arc<Mutex<Option<Box<HotplugtEventCallback>>>>,
}

pub struct HotplugEventHandler {
    evt_sender: std::sync::mpsc::Sender<HotplugEvent>,
    callback: Arc<Mutex<Option<Box<HotplugtEventCallback>>>>,
}

impl<T: UsbContext> rusb::Hotplug<T> for HotplugEventHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        let (vendor_id, product_id) = device
            .device_descriptor()
            .map(|d| (d.vendor_id(), d.product_id()))
            .unwrap_or((0, 0));
        let evt = HotplugEvent::DeviceArrived {
            vendor_id,
            product_id,
        };
        // let udev on linux settle down and assign permissions
        thread::sleep(std::time::Duration::from_millis(500));
        let _ = self.evt_sender.send(evt);
        if let Some(cb) = self.callback.lock().as_ref() {
            cb(evt)
        }
    }

    fn device_left(&mut self, device: Device<T>) {
        let (vendor_id, product_id) = device
            .device_descriptor()
            .map(|d| (d.vendor_id(), d.product_id()))
            .unwrap_or((0, 0));
        let evt = HotplugEvent::DeviceLeft {
            vendor_id,
            product_id,
        };

        let _ = self.evt_sender.send(evt);
        if let Some(cb) = self.callback.lock().as_ref() {
            cb(evt)
        }
    }
}

pub fn run_hotplug_detector<F: Fn(HotplugEvent) + Send + 'static>(
    callback: F,
) -> Result<HotplugDetector, anyhow::Error> {
    if rusb::has_hotplug() {
        let (sender, receiver) = std::sync::mpsc::channel::<HotplugEvent>();
        let context = Context::new()?;
        let detector = HotplugDetector {
            receiver,
            callback: Arc::new(Mutex::new(Some(Box::new(callback)))),
        };

        let cloned_callback = detector.callback.clone();
        thread::spawn(move || {
            let hotplug_result = HotplugBuilder::new().register(
                &context,
                Box::new(HotplugEventHandler {
                    evt_sender: sender,
                    callback: cloned_callback,
                }),
            );
            if let Err(e) = hotplug_result {
                log::error!("error registering hotplug handler: {:?}", e);
                return;
            }
            let reg: Box<rusb::Registration<Context>> = Box::new(hotplug_result.unwrap());
            loop {
                let result = context.handle_events(None);
                if result.is_err() {
                    log::error!("error handling libusb events: {:?}", result.err());
                    break;
                }
            }
            context.unregister_callback(*reg);
        });

        Ok(detector)
    } else {
        Err(anyhow::anyhow!("Hotplug not supported!"))
    }
}
