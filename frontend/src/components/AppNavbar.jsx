import React from 'react';
import { Navbar, Nav } from 'react-bootstrap';
import { Link } from 'react-router';
import InviteButton from './InviteButton';

const AppNavbar = () => {
	return (
		<Navbar bg="dark" variant="dark" expand="lg">
			<Navbar.Brand as={Link} to="/sm64-crypto">Mario 64 Crypto</Navbar.Brand>
			<Navbar.Toggle aria-controls="basic-navbar-nav" />
			<Navbar.Collapse id="basic-navbar-nav">
				<Nav className="ml-auto">
				<Nav.Link as={Link} to="/sm64-crypto/game">Game</Nav.Link>
				<Nav.Link as={Link} to="/sm64-crypto/explorer">Explorer</Nav.Link>
				<InviteButton/>
				</Nav>
			</Navbar.Collapse>
		</Navbar>
	);
};

export default AppNavbar;
