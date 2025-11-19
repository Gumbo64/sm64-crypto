import React from 'react';

const CopyButton = ({ copyText, children }) => {
    const handleCopy = () => {
        if (copyText) {
            navigator.clipboard.writeText(copyText)
                .catch(err => {
                    console.error('Failed to copy: ', err);
                });
        }
    };

    return (
        <button onClick={handleCopy}>
            {children}
        </button>
    );
};

export default CopyButton;
