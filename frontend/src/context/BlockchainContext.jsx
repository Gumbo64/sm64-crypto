import React, { createContext, useState, useEffect } from 'react';
import { getROM } from "sm64-binds-frontend";
import { BlockChainClientWeb } from "sm64-crypto-browser";

async function init_blockchain(name, ticket) {
    let rom_bytes = new Uint8Array(await getROM());
    return await BlockChainClientWeb.new(rom_bytes, name, ticket);
}

const BlockchainContext = createContext();

const BlockchainProvider = ({ children }) => {
	const [hasRom, setHasRom] = useState(false);
	const [blockchain, setBlockchain] = useState(null);

	useEffect(() => {
		const fn = async () => {
			if (hasRom && blockchain == null) {
				const name = new URLSearchParams(window.location.search).get('name') || prompt("Enter your username:");
				const ticket = new URLSearchParams(window.location.search).get('ticket') || prompt("Enter your ticket (or otherwise empty)");
				setBlockchain(await init_blockchain(name, ticket));
			}
		}
		fn();
	}, [hasRom]);

	return (
		<BlockchainContext.Provider value={{ hasRom, setHasRom, blockchain }}>
		{children}
		</BlockchainContext.Provider>
	);
};

export { BlockchainContext, BlockchainProvider };
