

# SM64 Crypto

## Summary
<!-- Fill in a brief summary of your project here. Describe its purpose, features, and any other relevant information. -->
This project manages a blockchain (perhaps a cryptocurrency in future) where instead of mining, players play Mario 64 to create blocks. At the moment it has the sm64 mining/evaluating, the blockchain itself, networking, and consensus.

## Usage
To use the project, follow the installation, cd into `prod` and then run `./main` to see the commands


## Installation
Before you begin, ensure you have the following:
- **SM64 z64**: Ensure that you legally obtain a US copy of the game as a z64 file.

The following instructions are based off the [sm64-port repository](https://www.github.com/sm64-port/sm64-port)

### Windows steps
1. download this repository as zip
2. extract it
3. click and run `windows_install.bat`
4. do what it says
5. after it is done, click `windows_play_game.bat`

### Linux steps

1. download this repository as zip
2. extract it
3. right click and run `install.sh` (or cd into the root of the repository and run it in the terminal)
4. do what it says
5. after it is done, run `play_game.sh` to play


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