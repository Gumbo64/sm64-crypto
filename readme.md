# SM64 Crypto
## Summary
Creating an independent cryptocurrency gained exclusively by playing Mario 64. 
Since gameplay *directly* seals blocks, it has many unique properties: 
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
- [ ] Browser Version
- [ ] Wallets
- [ ] Transactions


## Building
Before you begin, ensure you have the following:
- **SM64 z64**: Ensure that you legally obtain a US copy of the game as a z64 file. It should be 8.00MB large, and put in the main directory (next to this readme)

### Building


## Credits
* Iroh
* sm64-port
