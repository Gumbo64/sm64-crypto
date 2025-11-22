import React, { useContext, useState, useEffect } from 'react';
import { BlockchainContext } from '../context/BlockchainContext';

const InviteButton = () => {
    const { blockchain } = useContext(BlockchainContext);
    const [myInvite, setMyInvite] = useState(null);

    // Set the invite when the blockchain starts
    useEffect(() => {
        if (blockchain) {
            const fn = async () => {       
                const url = new URL(window.location.href);
                url.search = '';
                url.searchParams.set("ticket", await blockchain.get_ticket());
                setMyInvite(url);
            }
            fn();
        }
    }, [blockchain]);

    const handleCopy = () => {
        if (myInvite) {
            navigator.clipboard.writeText(myInvite)
                .catch(err => {
                    console.error('Failed to copy: ', err);
                });
        }
    };

    return (
        <button onClick={handleCopy} disabled={!myInvite}>
            Copy Invite Link
        </button>
    );
};

export default InviteButton;
