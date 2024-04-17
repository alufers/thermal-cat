# thermal-cat

![Screenshot](./docs/screenshot.png)

A GUI viewer application for the Infiray P2 Pro thermal camera.

While the project is still a Work in Progress, binaries for Windows and Linux can be downloaded from the "Actions" tab: https://github.com/alufers/thermal-cat/actions

# Running from source

Clone the repository and run:

```sh
cargo run
```

# Roadmap

- [x] Connecting and decoding thermal data from the camera
- [x] Adding markers to the image, min max markers
- [x] Rotating and zooming
- [x] Changing color gradients
- [x] Automatic range
- [x] Histogram of temperatures in frame
- [x] Chart of marker temperature vs. time
- [x] Celcius/Fahrenheit switching
- [ ] Capturing images
- [ ] Capturing video
- [ ] Area measurement
- [ ] Line measurement
- [ ] Triggering camera calibration
- [ ] Proper macOS support

While this project might seem stalled for the time being, I am actively working on a customized UVC driver for the camera in a separate project (private for now). It will enable me to send custom commands to the camera which trigger the calibration. Also it will fix macOS support, because the default macOS driver seems to be converting YUV422 data from the camera to a different color format, which unfortunately adds some nasty artifacts to the temperature data. 



License: GPL-3.0
