import React, { createContext, useState, useEffect } from 'react';
import { getROM } from "sm64-binds-frontend";
import { BlockChainClientWeb } from "sm64-crypto-browser";

const BlockchainContext = createContext();

const BlockchainProvider = ({ children }) => {
	const [hasRom, setHasRom] = useState(false);
	const [blockchain, setBlockchain] = useState(null);

	const init_blockchain = async (name, ticket) => {
		if (blockchain != null) {
			console.log("Reinitialising blockchain!")
		}

		let rom_bytes = new Uint8Array(await getROM());
		setBlockchain(await BlockChainClientWeb.new(rom_bytes, name, ticket));
	}

	useEffect(() => {
		if (hasRom && blockchain == null) {
			const name = new URLSearchParams(window.location.search).get('name') || prompt("Enter your username:");
			const ticket = new URLSearchParams(window.location.search).get('ticket') || prompt("Enter your ticket (or otherwise empty)");
			init_blockchain(name, ticket);
		}

	}, [hasRom, blockchain]);

	return (
		<BlockchainContext.Provider value={{ hasRom, setHasRom, blockchain, init_blockchain }}>
		{children}
		</BlockchainContext.Provider>
	);
};

export { BlockchainContext, BlockchainProvider };
