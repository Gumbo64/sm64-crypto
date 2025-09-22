

# SM64 Crypto

## Summary
This project manages a blockchain (perhaps a cryptocurrency in future) where instead of mining, players play Mario 64 to create blocks. At the moment it has the sm64 mining/evaluating, the blockchain itself, networking, and consensus.

## Controls
Keyboard button - N64 Button

WASD - Joystick
I - A
J - B
O - Z
Enter - Start

Special keys
R - Go back ~4 seconds in the run
Q - Completely restart run
hold Space - Change to 0.1x Speed
hold (Shift + Space) - Change to 10x Speed (or as fast as your computer can go)

## Installation
Before you begin, ensure you have the following:
- **SM64 z64**: Ensure that you legally obtain a US copy of the game as a z64 file.

### Windows steps
1. download this repository as zip
2. extract it
3. Put your sm64 ROM into the root of the extracted folder, and call it `baserom.us.z64`
4. click and run `windows_install.bat`
5. do what it says (it will take a while)
6. after it is done, click `windows_play_game.bat`

### Linux steps
1. download this repository as zip
2. extract it
3. Put your sm64 ROM into the root of the extracted folder, and call it `baserom.us.z64`
4. right click and run `install.sh` (or cd into the root of the repository and run it in the terminal)
5. do what it says (it will take a while)
6. after it is done, run `play_game.sh` to play

### Podman Desktop (no GUI or mining)
1. Put baserom.us.z64 in the root folder (next to readme.md etc)
2. On podman desktop, go Containers -> Create -> Containerfile or Dockerfile (purple)
3. Select Containerfile path by navigating to this folder, then selecting Dockerfile
4. Name it sm64-crypto optionally
5. Click Build and wait for it to build
6. Once it's done, go to Images and click the icon next to the sm64-crypto image that looks like a play button
7. go to the bottom and click Start Container


## Credits
* Iroh
* sm64-port