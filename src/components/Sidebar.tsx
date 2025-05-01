import React from 'react';
import { Link } from 'react-router-dom';
import { FaHome, FaCog } from 'react-icons/fa';
import './Sidebar.css';

const Sidebar: React.FC = () => {
  return (
    <nav className="sidebar">
      <ul>
        <li>
          <Link to="/">
            <FaHome />
            <span>Home</span>
          </Link>
        </li>
        <li>
          <Link to="/settings">
            <FaCog />
            <span>Settings</span>
          </Link>
        </li>
      </ul>
    </nav>
  );
};

export default Sidebar; 