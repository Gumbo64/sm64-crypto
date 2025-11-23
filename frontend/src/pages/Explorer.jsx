import { useEffect } from "react";
import { useRef, useState, useContext } from "react";

import BlockWindow from "../components/BlockWindow";
import { BlockchainProvider, BlockchainContext } from '../context/BlockchainContext';

function Explorer() {  
    const { blockchain } = useContext(BlockchainContext);

	const [blocks, setBlocks] = useState(null);

	const refreshBlocks = async () => {       
		if (blockchain) {
			let head_hash = await blockchain.get_head_hash()

			let b = await blockchain.get_block(head_hash);
			let block_array = [b];
			while (true) {
				console.log(b.prev_hash);
				try {
					b = await blockchain.get_block(b.prev_hash);
					block_array.push(b);
				} catch (error) {
					break
				}
			}
			setBlocks(block_array);
		}
	}

	useEffect(() => {
		refreshBlocks();
	}, [blockchain]);

	return (
		<>
			<button onClick={refreshBlocks}>Refresh Blocks</button>
			{blocks && blocks.map((block, index) => (
				<BlockWindow key={index} block={block} />
			))}
			{!blocks && <h1>No blocks, or currently syncing</h1>}
		</>
	);
}

export default Explorer;
