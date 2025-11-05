import React, { useState, useEffect } from 'react';
import { calculateFileHash, isRomCached, updateWASMs, storeROM } from '../scripts/fileUpload.js'
import Loading from './Loading.jsx';

const FileUpload = ({ setHasRom }) => {
    const [loading, setLoading] = useState(true);
    const [file, setFile] = useState(null);
    const [error, setError] = useState(null);

    // Directly set the expected ROM hash
    const expectedHash = '9bef1128717f958171a4afac3ed78ee2bb4e86ce';

    const handleFileChange = async (event) => {
        const romFile = event.target.files[0];
        if (romFile) {
            const romBuffer = await romFile.arrayBuffer();
            setFile(romFile);
            setError(null);

            // Calculate the SHA-1 hash of the file
            const hash = await calculateFileHash(romBuffer);
            if (hash === expectedHash) {
                setLoading(true);
                await storeROM(romBuffer);
                setHasRom(true);
            } else {
                setError(`Hash mismatch: expected ${expectedHash}, but got ${hash}.`);
            }
        }
    };
  
    // if we already uploaded previously then skip
    useEffect(() => {
        isRomCached().then(async () => {
            await updateWASMs();
            setHasRom(true);
        }).catch(async () => {
            setLoading(false);
        })
    }, []);

    if (loading) {
        return (
            <Loading/>
        );
    }

    return (
        <>
            <main>
                <div>
                    <h1>Upload Super Mario 64 US (.z64, 8.00MB)</h1>
                    <input type="file" accept=".z64" onChange={handleFileChange} />
                    {error && <p style={{ color: 'red' }}>{error}</p>}
                    <p>Expected Hash (SHA-1): {expectedHash}</p>
                </div>
            </main>
        </>
    );
};

export default FileUpload;
