# SAKS hat for raspberry_pi

SAKS = swiss army knife shield

---

## my raspberry_pi notes

### cli tools

#### cpu temperature

> vcgencmd measure_temp

#### remap caps to backspace

setxkbmap -option caps:backspace

### setup raspberry_pi

How to connect raspberry_pi without internet/display/GUI Desktop? 

1. put empty ssh file to SD card root directory in order to enable SSH on first time boot
2. SSH to pi via ethernet cable or serial(RS-232 with RXD+TXD+GND, don't connect 5V pin!) port
