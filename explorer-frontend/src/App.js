import React from 'react';
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import Dashboard from './pages/Dashboard';
import BlockDetail from './pages/BlockDetail';
import TransactionDetail from './pages/TransactionDetail';
import AddressDetail from './pages/AddressDetail';
import BlocksList from './pages/BlocksList';
import Search from './components/Search';

function App() {
  return (
    <Router>
      <div className="App">
        <header className="header">
          <div className="header-content">
            <Link to="/" style={{ textDecoration: 'none' }}>
              <div className="logo">⚛️ Axiom Explorer</div>
            </Link>
            <nav className="nav">
              <Link to="/">Dashboard</Link>
              <Link to="/blocks">Blocks</Link>
              <a href="https://github.com/Ghost-84M/Axiom-Protocol" target="_blank" rel="noopener noreferrer">
                GitHub
              </a>
            </nav>
          </div>
        </header>

        <div className="container">
          <Search />
          
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/blocks" element={<BlocksList />} />
            <Route path="/block/:id" element={<BlockDetail />} />
            <Route path="/transaction/:hash" element={<TransactionDetail />} />
            <Route path="/address/:address" element={<AddressDetail />} />
          </Routes>
        </div>
      </div>
    </Router>
  );
}

export default App;
