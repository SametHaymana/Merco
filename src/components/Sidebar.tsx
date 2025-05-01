import React from 'react';
import { Link } from 'react-router-dom';
import { FaHome, FaCog, FaBars, FaTimes } from 'react-icons/fa';
import './Sidebar.css'; // We'll create this file next

interface SidebarProps {
  isOpen: boolean;
  toggleSidebar: () => void;
}

const Sidebar: React.FC<SidebarProps> = ({ isOpen, toggleSidebar }) => {
  return (
    <>
      <button className="sidebar-toggle" onClick={toggleSidebar}>
        {isOpen ? <FaTimes /> : <FaBars />}
      </button>
      <nav className={`sidebar ${isOpen ? 'open' : ''}`}>
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
    </>
  );
};

export default Sidebar; 