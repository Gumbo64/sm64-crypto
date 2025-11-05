import React from 'react';
import './Loading.css'; // Import the CSS file for styling

const Loading = () => {
    return (
        <div className="loader">
            <div className="spinner"></div>
            <p>Loading ROM...</p>
        </div>
    );
};

export default Loading;
