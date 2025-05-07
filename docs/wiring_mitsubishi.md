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



Mitsubishi Differential Pairs:
- White/Green and White/Blue A 
- Green and Blue B
- White/Orange and White/Brown C

el6021 Differential Pairs
- Pin 5 and 6 of el6021 connect with Pair B
- Pin 1 and 2 of el6021 connect with Pair A
- Pin 3 and 7 of el6021 connect with Pair C


What needs to be bridged?
- pin 5 and 6 of el6021 need to be bridged with Ethernet pin 4 and 6
- pin 1 and 2 of el6021 need to be bridged with Ethernet pin 3 and 5
- pin 3 and 7 of el6021 need to be bridged with Ethernet pin 1 and 7
