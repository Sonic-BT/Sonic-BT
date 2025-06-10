![image](https://github.com/user-attachments/assets/ede86f11-f39a-4a40-972c-a5aaf5fcd726)

# Sonic^{BT} Project Brief
For a while I have been wanting a bluetooth reciver for my wired earphones.
When looking for pre-existing options on the market it looked like i was going to spend $300+.
So I thought surley I can do that my self for cheaper and here we are.
I have a few goals for this project learn, remove outside dependacys (mobile apps, cloud...), amazing user experince or UX.

# Hardware Choices
### Brains ESP32-C6
The brains of the of the system managing all perifreals.
Why C series ESP32 I have selected this due to its risk-v archtexture.
Risk-v leads to lower power draw wich is crusial to battery operated system.
I had some on hand aswell making a easy desion.
### Audio Feasycom BT1058
This is a bluetooth enabled system on module or SOM resposible for reciving bluetooth audio signal.
BT1058 module features the latest qcc5181 from qualcomm feature packed to the brim.
Including highfedality codecs like LDAC.
This module is controlled over UART with commands listed in [documentation](https://document.feasycom.com/docs/audio/BT1058_EN/latest/) these are excuted by the ESP32-C6 to configure the module.

# Project Plan/Progress
To view the plan you can view the GitHub project or issue page.


|GitHub Project|[https://github.com/orgs/Sonic-BT/projects/1](Project)
|-|-|
|**GitHub Issues**|**[https://github.com/Sonic-BT/Sonic-BT/issues](Issue)**|
