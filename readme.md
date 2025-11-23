# SM64 Blockchain
## Summary

Eventually creating an independent cryptocurrency gained exclusively by playing Mario 64. 
Since gameplay *directly* seals blocks, it would have many unique properties: 
* No need for the initial investments, PVP or gas fees that required for other play-to-earn cryptos require
* Only mined by humans at the moment, otherwise it could be an open-invite benchmark for AI
* Produces a usable dataset of gameplay as the blockchain grows.

To successfully finish your gameplay, you must obtain 1 star. Therefore you must to go to the top of bobomb battlefield's mountain and defeat King Bobomb.

Unique problems require unique solutions, so some mechanisms have been invented:
* While playing, your button inputs are randomly perturbed (like poking a robot) to prevent hard-coded or recycled gameplay.
* The random seed is calculated based on the details of the block, so the RNG is verifiable and the gameplay is linked to the block
* Instead of pre-computing all the random pertubations at the beginning, the RNG factors in Mario's current position and velocity after each frame of gameplay (unpredictable). This prevents players from abusing compute power to cherrypick hassle-free seeds.

Progress:
- [x] Mario 64 integration: Playing and verifying
- [x] Block creation, storage, chain
- [x] P2P Networking, broadcasting blocks
- [x] Consensus and syncing
- [x] Calculating random seeds and random input generation
- [x] Browser Version
- [ ] Wallets
- [ ] Transactions


## Building
Before you begin, ensure you have the following:
- **SM64 ROM**: Ensure that you legally obtain a US copy of the game as a z64 file. It should be 8.00MB large, and put in the main directory (next to this readme)

### Native node
1. install cargo https://doc.rust-lang.org/cargo/getting-started/installation.html
2. Simply run `cargo run` in the root directory, or `cargo run -- -t <ticket>` if you're providing a ticket

### Web version
Get the ROM and then go to this link
https://gumbo64.github.io/sm64-crypto/

### Running your own web version
The webserver provides a static page (same as the link provided above), so you don't really need to do this unless you're modding (have fun!)
1. install cargo https://doc.rust-lang.org/cargo/getting-started/installation.html
2. run `cargo install wasm-pack wasm-bindgen-cli`
2. `cd frontend`
3. `npm run build:wasm:release`
4. `npm run dev`
5. click the link in the terminal


## Credits
* Iroh
* sm64-port
