use imgui::*;
use nokhwa::{utils::{CameraIndex, RequestedFormat, RequestedFormatType}, pixel_format::RgbFormat, Camera, native_api_backend, query};

mod support;

fn main() {
    let system = support::init(file!());

    let backend = native_api_backend().unwrap();
            let devices = query(backend).unwrap();
            println!("There are {} available cameras.", devices.len());


    let mut selected_device: usize = 0;

    system.main_loop(move |_, ui| {
        ui.window("Hello world")
            .size([300.0, 250.0], Condition::FirstUseEver)
            .build(|| {
               

             

                

                ui.separator();
                ui.combo("Select camera", &mut selected_device, &devices, | d | {
                  
                    return d.description().to_string().into();
                });
            });

            ui.window("Win2")
            .size([300.0, 250.0], Condition::FirstUseEver)
            .build(|| {
               

             

                

               
            });
    });
}
