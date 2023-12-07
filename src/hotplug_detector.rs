use std::thread;

use rusb::{Context, Device, HotplugBuilder, UsbContext};

pub enum HotplugEvent {
    DeviceArrived { vendor_id: u16, product_id: u16 },
    DeviceLeft { vendor_id: u16, product_id: u16 },
}

pub struct HotplugDetector {
    evt_sender: std::sync::mpsc::Sender<HotplugEvent>,
    callback: Box<dyn Fn(HotplugEvent) -> () + Send>,
}

impl<T: UsbContext> rusb::Hotplug<T> for HotplugDetector {
    fn device_arrived(&mut self, device: Device<T>) {
        let (vendor_id, product_id) = device
            .device_descriptor()
            .map(|d| (d.vendor_id(), d.product_id()))
            .unwrap_or((0, 0));
        let evt = HotplugEvent::DeviceArrived {
            vendor_id,
            product_id,
        };
        let _ = self.evt_sender.send(evt);
    }

    fn device_left(&mut self, device: Device<T>) {
        let (vendor_id, product_id) = device
            .device_descriptor()
            .map(|d| (d.vendor_id(), d.product_id()))
            .unwrap_or((0, 0));
        let _ = self.evt_sender.send(HotplugEvent::DeviceLeft {
            vendor_id,
            product_id,
        });
    }
}

pub fn run_hotplug_detector<F: Fn(HotplugEvent) -> () + Send + 'static>(
    callback: F,
) -> Result<std::sync::mpsc::Receiver<HotplugEvent>, anyhow::Error> {
    if rusb::has_hotplug() {
        let (sender, receiver) = std::sync::mpsc::channel::<HotplugEvent>();
        let context = Context::new()?;
        let reg: Box<rusb::Registration<Context>> =
            Box::new(HotplugBuilder::new().enumerate(true).register(
                &context,
                Box::new(HotplugDetector {
                    evt_sender: sender,
                    callback: Box::new(callback),
                }),
            )?);
        thread::spawn(move || {
            loop {
                let result = context.handle_events(None);
                if result.is_err() {
                    log::error!("error handling libusb events: {:?}", result.err());
                    break;
                }
            }
            context.unregister_callback(*reg);
        });

        Ok(receiver)
    } else {
        Err(anyhow::anyhow!("Hotplug not supported!"))
    }
}
