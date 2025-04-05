# Cannonball LL Build Manual (wireless version) ([日本語](https://note.com/taro_hayashi/n/n3da1e116aa8d))

![](img/IMG_0243.jpg)

## 1 Introduction

### 1.1 Attention

- Lithium polymer batteries can explode and cause injury or fire, so handle them with care when in use.
- You create the firmware yourself. The ZMK Firmware, which serves as a template, is available to the public.
  - https://github.com/Taro-Hayashi/zmk-config-th/tree/Cannonball-LL
- It is assumed that the user has some familiarity with soldering.

### 1.2 Contents

![](img/IMG_1136.jpg)
![](img/IMG_1130.jpg)

|     | Types                  | Quantities |               |
| --- | ---------------------- | ---------- | ------------- |
| 1   | PCB                    | 1          |               |
| 2   | Runner I               | 1          |               |
| 3   | Runner II              | 1          |               |
| 4   | Top cover              | 1          |               |
| 5   | Knob cover             | 1          |               |
| 6   | Short screws           | 2          | M2 6mm        |
| 7   | Long screws            | 4          | M2 10mm       |
| 8   | Nuts                   | 4          | M2            |
| 9   | Insert nuts            | 2          | M2 5mm or 6mm |
| 10  | Diodes                 | 24         | 1N4148        |
| 11  | Capacitors             | 2          | 0.1uF         |
| 12  | Shift registers        | 2          | 74HC595       |
| 13  | back buttons           | 3          | 3x6x4.3       |
| 14  | Side buttons           | 4          | 3x6x5         |
| 15  | Slide switch           | 1          | SK12F14       |
| 16  | Mouse buttons          | 2          | YD-003        |
| 17  | Lever buttons          | 2          | THMU27        |
| 18  | Rotary Encoder         | 1          | RKJXT1F42001  |
| 19  | Rubber feet            | 5          |               |
| 20  | Pin header             | 1          | 40pins        |
| 21  | Pin Sockets            | 2          | 14pins        |
| 22  | Hotswap Sockets        | 10         | CPG151101S11  |
| 23  | 1U Wheel Encoder       | 2          | THQWGD001C    |
| 24  | Micro Controller Board | 1          | RP2040-Zero   |

### 1.3 Additional required
![](img/IMG_1192.jpg)

| Types              | Quantities  |
| ---------------- | --- |
| Keyswitches      | 10  |
| Keycaps          | 10  |
| XIAO BLE | 1   |
| Lead wire | 1   |
| LiPo battery | 1   |
| Protective tape, rubber sheeting, etc.
 | 1   |

### 1.4 Further additional required to a wireless mouse
Capacitors and resistors are available in leaded or 1206 and 0805 packages.
The regulator can be TO92 or SOT23-5. The image shows the sensor set on BOOTH.

![](img/IMG_1149.jpg)

|    | Types    | Quantities  |                  | 
| -------- | ------- | --- | ---------------- | 
|          | Sensor & lens    | 1   | PMW3610          | 
| VR       | Regulator | 1   |  3.3V input 1.8 or 1.9V output  |
| C3、C4、C5 | Capacitor  | 3   | 0.1uF X7R        |
| C6、C7    |         | 2   | 0.01uF X7R       | 
| C8       |         | 1   | 10uF X7R         | 
| C9       |         | 1   | 3.3uF 16V        | 
| C in     |         | 1   | input capacitor      |
| C out    |         | 1   | output capacitor      | 
| R        | Resistor      | 1   | 10kΩ             |        
|          | Mouse sole  | 5   | 0.65mm and 0.8mm thickness confirmed|        

### 1.5 Optional

| Types                    | Quantitie  |                                                                                             |
| ---------------------- | --- | --------------------------------------------------------------------------------------------- |
| PH connector Post Side type 2P | For use with batteries with PH connector. Be sure to check the plus and minus. It must be a smaller size or there may be interference.
 |
| Choc V1/V2 keyswitches | 10  | Height can be reduced. Sockets and top plates are not used. It cannot coexist with MX switches.                                   |
| Tape or glue           | 1   | When using Choc switches, isolate the head of the pin header to prevent unexpected operation. |
| [3-keys blocker](stl/)           | 1   | Blocks the palm of the hand when held. |
| [Switch plate](stl/)           | 1   | 3DP switch plate with invisible pin header heads. |

### 1.6 Tools required
![](img/IMG_1469.jpg)

| Types                                                | 
| --------------------------------------------------- |
| Soldering iron                                               |
| Trowel                     |
| Soldering wire |
| Nipper                     | 
| Cutter                    |
| File                      | 
| Twezers                    |   
| screwdriver                 | 



### 1.7 Tools optional

| Types                               | 
| --------------------------------- | 
| C-type trowel tip                             | 
| flux|                    |
| IPA                               | 
| Heat-resistant mat                             | 
| Tester                              |
| Desoldering wire                         | 
| Design Knife                           |
| Masking tape                          | 


## 2 Preparation for assembly

### 2.1 Disconnect the mainboard

It can be split cleanly by cutting a slit with a cutter and bending it.
![](img/IMG_1160.jpg)
The tabs are cut away with nippers and the cut surfaces are sanded. Be careful not to remove the original board edges, as over-shaving will damage the circuitry.
![](img/IMG_1169.jpg)
This area can be left jagged without affecting the finished form.
![](img/IMG_1173.jpg)
It could be divided into three parts: main board #1, main board #2 and switch plate.
![](img/IMG_1176.jpg)

### 2.2 Remove parts from the runner

Runner I has 3 components and runner II has 17 components.
![](img/IMG_1181.jpg)

### 2.3 Remove the shift register and rotary encoder from the case

The shift register is pressed from the back with the runner II component (leverage). Be careful not to prick your finger with the pin.  
For the rotary encoder, cut the left and right claws with nippers.
![](img/IMG_1188.jpg)

### 2.4 Write test firmware

Download the uf2 file here.

- [Cannonball_LL-seeeduino_xiao_ble-zmk.uf2](https://github.com/Taro-Hayashi/Cannonball-LL/releases/download/0.28.2/Cannonball_LL-seeeduino_xiao_ble-zmk.uf2)

Once the XIAO BLE is connected to the PC via the USB cable, press the small reset button twice quickly.
![](img/IMG_1220.jpg)
Drag and drop the uf2 file as it is recognised as a XIAO-SENSE drive.
![](img/drive.jpg)
It will now be recognised as a keyboard.

### 2.5 Press fitting of insert nuts

Press-fit the insert nut into the printed item in the image. Main board #1 is also used.
![](img/IMG_1199.jpg)
Apply main board #1 to the backside and press the soldering iron onto the insert nut.
![](img/IMG_1204.jpg)
Ensure as few steps as possible with main board #1.
![](img/IMG_1212.jpg)
Cut out the reinforcement with nippers.
![](img/IMG_1218.jpg)

## 3 Soldering of main board #️1

- The pin headers that may come with the XIAO BLE are thinner than those included and should be discarded to avoid confusion.
- In this build manual, priority is given to checking the operation, and the mouse-ising components are put off until later, but it is easier to solder from the shortest components first.

### 3.1 Be careful when soldering

表面張力でパッドの上だけに溶けたはんだが広がるのをイメージします。フラックスを塗るととても簡単になります。
![](img/IMG_0654.jpg)
パッドや部品の足をよく熱して基板と部品両方に電気が通るようにします。
はんだが小さい穴（VIA）と繋がってしまわないように気をつけてください。
![](img/IMG_0668.jpg)
スルーホールの部品から出る足は2mm程度になるように気をつけます。2.5mmを超えると底面の3Dプリント品と干渉することがあります。
差し込んだ後にマスキングテープで固定してはんだ付けをすると楽になります。
![](img/IMG_0647.jpg)
また、3Dプリント品の融点は200度前後です。直接はんだごてが当たると溶けてしまうので気をつけます。

### 3.2 XIAO BLEのはんだ付け

5V（VUSB)、GP0の位置で裏表を確認してはんだ付けします。
![](img/IMG_1228.jpg)
裏面のRESET（1）とBAT+（2）のパッドを配線します。
RESETのパッドは剥がれるとリセットボタンも使えなくなってしまうので配線材を引っ張らないように気をつけるか、はんだ付けしないことも検討してください（背面ボタンでのリセットはできなくなります）。
![](img/IMG_1230.jpg)
（オプション）マウス化する場合はNFCの二つも配線します。
![](img/IMG_1233.jpg)
ランナーIの部品と干渉しないように気をつけます。
![](img/IMG_1237.jpg)
### 3.3 ダイオード、シフトレジスターのはんだ付け
D1からD7、SR1、SR2にマークを目印にしてはんだ付けします。
![](img/IMG_1241.jpg)

### 3.4 コンデンサーのはんだ付け
C1とC2にはんだ付けします。104と書いてあります。センサーセットのものは少し小さく特性が違うので間違えないように気をつけます。
![](img/IMG_1246.jpg)
### 3.5 側面ボタンとスライドスイッチのはんだ付け
側面ボタン4つをSS1からSS4に、スライドスイッチ1つをSLに差し込んではんだ付けします。
![](img/IMG_1249.jpg)

### 3.6 背面ボタンのはんだ付け
背面ボタンは裏側から差し込みます。
![](img/IMG_1253.jpg)
### 3.7 動作の確認
USBケーブルでPCに接続して側面ボタンと、背面ボタンのうち並んだ2つの動作を確認します。
![](img/IMG_1256.jpg)
残りの背面ボタンを2回すばやく押してXIAO-SENSEドライブが出てくることを確認します。ファームウェアの更新の際に使います。
![](img/IMG_1259.jpg)
確認をしたらケーブルを外します。スライドスイッチはバッテリーのオンオフに使われます。
### 3.8  （オプション）マウス化用部品のはんだ付け
白枠内にレギュレーターをVR、コンデンサーをC3からC9、Cin、Cout、抵抗をRにはんだ付けします。
![](img/IMG_1263.jpg)
C9の3.3uF 16Vのコンデンサーには極性があるので確認してからはんだ付けします。
![](img/IMG_1268.jpg)
センサーセットのレギュレーターは平面が下になります。
![](img/IMG_1272.jpg)
リードタイプの部品ははんだ付けの後に寝かせます。
![](img/IMG_1273.jpg)
センサーをはんだ付けします。シルク印刷と小さいマークを合わせます。
![](img/IMG_1281.jpg)
裏面のシールを剥がしてレンズをのせ、表面の足を溶かして軽く固定します。
![](img/IMG_1286.jpg)
![](img/IMG_1291.jpg)
ランナーIの部品を合わせてPCに接続して、センサーに反応があることを確認します。
![](img/IMG_1292.jpg)
動かない場合はコンデンサーが間違っていたり足がブリッジしていないか確認してみてください。
作業中はレンズを傷つけないように気をつけます。
### 3.9 ピンソケットのはんだ付け
ピンソケット、ピンヘッダーを3ピンずつ8つ切り出します。
![](img/IMG_0864.jpg)
ピンソケットにピンヘッダーの短い方を差し込んで、ピンソケット側をメインボード#1に配置します。
![](img/IMG_1302.jpg)
ランナーIIの部品を6つ差し込みます。入りにくい場合はピンヘッダーやピンソケットの切れ跡を削ってください。
![](img/IMG_1479.jpg)
メインボード#2をピンヘッダーに合わせてかぶせます（まだはんだ付けしません）。全てのピンヘッダーを通すのが難しい時は無理をして基板を擦らないように数個ずつにします。
![](img/IMG_1306.jpg)
裏返したらメインボード#1にピンソケットをはんだ付けします。
![](img/IMG_1311.jpg)
はんだ付けが終わったら表に戻してメインボード#2を外します。
プリント品を外す必要はありません。
![](img/IMG_1479.jpg)
メインボード#2をはんだ付けしてしまっていた場合はテコを使って外してください。ピンヘッダが曲がると接触不良の原因となるため垂直に引き抜くことを意識してください。
![](img/IMG_1433.jpg)
テスターがある場合はXIAO BLE、シフトレジスターのGPIOとピンヘッダーが導通していることを確認します。
![](img/IMG/test.jpg)

### 3.10 バッテリーのはんだ付け
スライドスイッチが画像と同じ状態になっていることを確認してバッテリーをはんだ付けします。
![](img/IMG_1315.jpg)
必ず＋と−を確認してください。はんだ付けが終わったらスライドスイッチをオンにしてBluetooth対応機器と接続できるか確かめます。

起動しているのに接続に難がある場合は一度こちらのファームウェアでリセットしてからテストファームウェアを入れ直してください。
- [settings_reset-seeeduino_xiao_ble-zmk.uf2](https://github.com/Taro-Hayashi/Cannonball-LL/releases/download/0.28.2/settings_reset-seeeduino_xiao_ble-zmk.uf2)
## 3.11 下半分の組み立て
ランナーIの部品の小さな穴にRP2040-Zeroのリセットボタン用パーツを溶接します。
![](img/IMG_1323.jpg)
ランナーIの部品を組み合わせ、ランナーIIの部品を5個置きます。中央の部品の向きに気をつけます。
![](img/IMG_1328.jpg)

ここに乗せるボタンのストッパーの角が干渉するのでニッパーで少し斜めに切ります。
![](img/IMG_1440.jpg)

上からメインボード#1を置き、ランナーIの角の部品の一つにナットを取り付けます。
![](img/IMG_1331.jpg)
合計9個のランナーIの部品と4つのナットを取り付けました。
![](img/IMG_1335.jpg)
裏面にマウスソールを貼り、安全な場所に保管します。
![](img/IMG_1342.jpg)
## 4 メインボード#2のはんだ付け

### 4.1 ダイオードのはんだ付け
D8からD24までをはんだ付けします。D19以降は取り付ける面が違います。
![](img/IMG_1344.jpg)
![](img/IMG_1346.jpg)
### 4.2 MXソケットのはんだ付け
S1からS10まではんだ付けします。
![](img/IMG_1347.jpg)
### 4.3  レバーボタンのはんだ付け
L1、L2に取り付けます。側面にも接点があるのではんだ付けします。
![](img/IMG_1354.jpg)
### 4.4 マウスボタンのはんだ付け
M1、M2に取り付けます。クリックバーの位置をシルク印刷に合わせます。
![](img/IMG_1363.jpg)
### 4.5  1Uホイールエンコーダーの組み立て
小袋の部品のうち以下のものを使用します。
![](img/IMG_1369.jpg)

|     | 部品名        | 個数  |               |
| --- | ---------- | --- | ------------- |
| 1   | ホイール       | 2   |               |
| 2   | シャフト       | 2   |               |
| 3   | スペーサー      | 2   |               |
| 4   | ベース        | 2   |               |
| 5   | カバー        | 2   |               |
| 6   | ホイールエンコーダー | 2   | 6x6x7         |
| 7   | クリックボタン    | 2   | 11mm          |
| 8   | ネジ（小）      | 4   | M2 5mm or 6mm |
| 9   | ナット        | 4   | M2            |

E1とE2に、方向を選んでクリックボタンとホイールエンコーダーをはんだ付けします。ビルドガイドでは画像の向きにしました。
![](img/IMG_1370.jpg)
ホイールの平らな面からシャフトを差し込みます。
![](img/IMG_1378.jpg)
エンコーダーにホイールを取り付けます。六角形の穴と六角形の軸が合うようにします。
![](img/IMG_1379.jpg)
スペーサーを取り付けてタクトスイッチに乗せます。
![](img/IMG_1386.jpg)
ベースをネジとナットで取り付けます。
![](img/IMG_1389.jpg)
カバーを取り付けます。
![](img/IMG_1394.jpg)
### 4.6 ロータリーエンコーダーのはんだ付け
STにはんだ付けします。
![](img/IMG_1396.jpg)
### 4.7 トップカバーの取り付け
裏面からネジ（短）で取り付けます。
![](img/IMG_1397.jpg)
マウスボタンがクリック可能なことを確認します。
![](img/IMG_1400.jpg)
### 4.8 ピンヘッダのはんだ付け
改めてメインボード#2の部品の向き、はんだ付けの状態を確認します。
メインボード#2の裏面の、バッテリーが当たる部分をテープやゴムシートなどで保護します。
![](img/IMG_1420.jpg)
メインボード#1に乗せてピンヘッダをはんだ付けします。
![](img/IMG_1412.jpg)
隙間をできるだけ減らすようにしながら角のピンヘッダから一つずつ作業をすると綺麗に出来上がります。
### 4.9 動作確認
ロータリーエンコーダーにランナーIについていたシャフトを取り付けます。
![](img/IMG_1415.jpg)
USBケーブルでPCに接続して動作を確認します。キースイッチを一つ使ってソケットの動作を確認していきます。キースイッチの足が折れないように気をつけてください。
![](img/IMG_1426.jpg)
動作を確認したらUSBケーブルを取り外します。
### 4.10 スイッチプレートを取り付ける
メインボード#2とスイッチプレートの位置を合わせてキースイッチを取り付けてゆきます。
![](img/IMG_1430.jpg)
The keycap and knob cover are completed when fitted.
![](img/IMG_1444.jpg)

My ZMK repository is here.
- https://github.com/Taro-Hayashi/zmk-config-th/tree/Cannonball-LL

## 5 Misc

### 5.1 Maintenance

If the knob becomes loose or the 1U wheel encoder cover comes off easily, it can be adjusted and fixed by melting it with a soldering iron.  
If the top and bottom fastenings become loose or poor contact, consider replacing the pin headers and pin sockets.

### 5.2 About unused pads

These pads are connected to the LEDs and the centre push.
![]()  

### 5.3 Data for printing

- [stl](/stl)

Optional or colour-changed parts can be printed.
![](img/IMG_1892.jpg)  
  
### 5.5 Thanks

I referred to snize([@snize](https://x.com/snize))'s SEIBOKU for the selection of components and the firmware code.

- https://github.com/snize/BOB-PMW3610-SEIBOKU
  
### 5.6 Sales website

- BOOTH - [https://tarohayashi.booth.pm/items/6736492](https://tarohayashi.booth.pm/items/6736492)

