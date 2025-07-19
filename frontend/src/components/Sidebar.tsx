import React, { useState } from 'react';
import { motion, AnimatePresence, type Variants } from 'framer-motion';
import { FiHome, FiServer, FiSettings, FiChevronLeft, FiChevronRight, FiLogOut, FiBox } from 'react-icons/fi';

const Sidebar = () => {
  const [isOpen, setIsOpen] = useState(true);

  const toggleSidebar = () => {
    setIsOpen(!isOpen);
  };

  const sidebarVariants: Variants = {
    open: { width: '250px', transition: { duration: 0.3, ease: "easeInOut" } },
    closed: { width: '80px', transition: { duration: 0.3, ease: "easeInOut" } },
  };

  const itemVariants: Variants = {
    open: { opacity: 1, x: 0, display: 'flex', transition: { duration: 0.3, ease: "easeInOut" } },
    closed: { opacity: 0, x: -10, transitionEnd: { display: 'none' } },
  };

  const iconVariants = {
    open: { marginRight: '1rem' },
    closed: { marginRight: '0' },
  };

  return (
    <motion.div
      variants={sidebarVariants}
      animate={isOpen ? 'open' : 'closed'}
      className="bg-white text-gray-800 h-screen p-4 flex flex-col justify-between shadow-lg border-r border-gray-200"
    >
      <div>
        <div className="flex items-center justify-between mb-8">
          <AnimatePresence>
            {isOpen && (
              <motion.div
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                className="flex items-center"
              >
                <FiBox className="text-3xl text-indigo-600 mr-2" />
                <h1 className="text-2xl font-bold">Eclipta</h1>
              </motion.div>
            )}
          </AnimatePresence>
          <button onClick={toggleSidebar} className="text-2xl p-1 rounded-full hover:bg-gray-200 transition-colors">
            {isOpen ? <FiChevronLeft /> : <FiChevronRight />}
          </button>
        </div>
        <ul>
          {[
            { icon: FiHome, text: 'Dashboard', path: '/dashboard' },
            { icon: FiServer, text: 'Agents', path: '/agents' },
            { icon: FiSettings, text: 'Settings', path: '/settings' },
          ].map((item, index) => (
            <li key={index} className="mb-4">
              <a href={item.path} className="flex items-center p-2 hover:bg-gray-100 rounded-lg transition-colors">
                <motion.div variants={iconVariants} animate={isOpen ? 'open' : 'closed'}>
                  <item.icon className="text-2xl" />
                </motion.div>
                <AnimatePresence>
                  {isOpen && <motion.span variants={itemVariants} initial="closed" animate="open" exit="closed">{item.text}</motion.span>}
                </AnimatePresence>
              </a>
            </li>
          ))}
        </ul>
      </div>
      <div>
        <a href="#" className="flex items-center p-2 hover:bg-gray-100 rounded-lg transition-colors">
          <motion.div variants={iconVariants} animate={isOpen ? 'open' : 'closed'}>
            <FiLogOut className="text-2xl" />
          </motion.div>
          <AnimatePresence>
            {isOpen && <motion.span variants={itemVariants} initial="closed" animate="open" exit="closed">Logout</motion.span>}
          </AnimatePresence>
        </a>
      </div>
    </motion.div>
  );
};

export default Sidebar;