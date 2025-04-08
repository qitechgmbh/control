| Color Code   | Ethernet Bezeichnung | Ethernet Pin | Mitsubishi Bezeichnung | Mitsubishi Pin | Mitsubishi Name |
| ------------ | -------------------- | ------------ | ---------------------- | -------------- | --------------- |
| WHITE/ORANGE | TX+(B)               | 1            | SG (ground)            | 1              | SG              |
| ORANGE       | TX- (B)              | 2            | PU Power Supply        | 2              | -               |
| White/Green  | RX+ (B)              | 3            | Inverter receive+      | 3              | RDA             |
| Blue         | Unused/POE (B)       | 4            | Inverter Send-         | 4              | SDB             |
| White/Blue   | Unused/POE (B)       | 5            | Inverter Send+         | 5              | SDA             |
| Green        | RX- (B)              | 6            | Inverter receive -     | 6              | RDB             |
| White/Brown  | Unused/POE (B)       | 7            | SG (ground)            | 7              | SG              |
| Brown        | Unused/POE (B)       | 8            | PU power supply        | 8              | -               |

we need pin 3 4 5 6 and 7
Both Grounds always connect to Ground (Brown Jumper)

Combinations:
Green and Blue, White/Green and White/Blue

Gnd von usb Chip zu SG und SG von Inverter
A goes to greend and blue pair
B goes to green/white and blue/white
