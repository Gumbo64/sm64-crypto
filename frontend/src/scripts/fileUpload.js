import sm64XOR from '../assets/pkg/sm64.us.wasm.xor'
import sm64_HEADLESSXOR from '../assets/pkg/sm64_headless.us.wasm.xor'

// import sm64WASM from '../assets/pkg/sm64.us.wasm?url'
// import sm64WASM_HEADLESS from '../assets/pkg/sm64_headless.us.wasm?url'

async function calculateFileHash(arrayBuffer) {
    const hashBuffer = await crypto.subtle.digest('SHA-1', arrayBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    return hashHex;
};

async function xorForWASM(key) {
    await xorForWASMSingle(sm64XOR, "sm64.us.wasm", key);

    await xorForWASMSingle(sm64_HEADLESSXOR, "sm64_headless.us.wasm", key);
}

async function xorForWASMSingle(wasmXOR, filename, key) {
    // Read and XOR the input file
    const inputFileResponse = await fetch(wasmXOR, { credentials: 'same-origin' });

    if (!inputFileResponse.ok) {
        throw new Error(`HTTP error! Status: ${inputFileResponse.status}`);
    }

    const keyBuffer = new Uint8Array(key)
    const inputFileBuffer = await inputFileResponse.arrayBuffer();
    const inputFileData = new Uint8Array(inputFileBuffer);
    const outputFileData = new Uint8Array(inputFileData.length);

    for (let i = 0; i < inputFileData.length; i++) {
        outputFileData[i] = inputFileData[i] ^ keyBuffer[i % keyBuffer.byteLength];
    }
    // Initialize IndexedDB
    const db = await openDatabase();
    const transaction = db.transaction('files', 'readwrite');
    const store = transaction.objectStore('files');
    const request = store.put({ id: filename, data: outputFileData });

    return new Promise((resolve, reject) => {
        request.onsuccess = (event) => {
            console.log(`${filename} has been saved to IndexedDB.`);
            resolve();
        };
        request.onerror = (event) => {
            console.error('Error saving to IndexedDB:', event);
            reject();
        };
    });
};

async function instantiateWasmSM64(info, func) {
    const wasmBuffer = await getWASM("sm64.us.wasm");
    const instance = await WebAssembly.instantiate(wasmBuffer, info);
    func(instance["instance"], instance["module"]);
}
async function instantiateWasmSM64_HEADLESS(info) {
    const wasmBuffer = await getWASM("sm64_headless.us.wasm");
    const instance = await WebAssembly.instantiate(wasmBuffer, info);
    func(instance["instance"], instance["module"]);
}

async function getWASM(filename) {
    const db = await openDatabase();
    const transaction = db.transaction('files', 'readonly');
    const store = transaction.objectStore('files');
    const request = store.get(filename);

    return new Promise((resolve, reject) => {
        request.onsuccess = (event) => {
            const result = event.target.result;
            if (result) {
                // Create a Blob from the stored data
                const blob = new Blob([result.data], { type: 'application/octet-stream' });
                
                // Read the Blob as an ArrayBuffer
                const reader = new FileReader();
                reader.onload = () => {
                    resolve(reader.result); // Resolve with the ArrayBuffer
                };
                reader.onerror = () => {
                    reject('Error reading the WASM Blob');
                };

                reader.readAsArrayBuffer(blob); // Convert Blob to ArrayBuffer
            } else {
                reject('File not found in IndexedDB');
            }
        };

        request.onerror = () => {
            reject('Error retrieving file from IndexedDB: ' + request.error);
        };
    });
}
async function isSM64Cached() {
    const request = await getWASM("sm64.us.wasm");
    return request !== undefined;
}

// This function opens the IndexedDB database to be used in locateFile function
async function openDatabase() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open('wasmDB', 1);

        request.onupgradeneeded = (event) => {
            const db = event.target.result;
            db.createObjectStore('files', { keyPath: 'id' });
            // resolve(db);
        };
        request.onsuccess = (event) => {
            resolve(event.target.result);
        };
        request.onerror = (event) => {
            reject(event.target.error);
        };
    });
}


export {calculateFileHash, xorForWASM, instantiateWasmSM64, instantiateWasmSM64_HEADLESS, isSM64Cached};