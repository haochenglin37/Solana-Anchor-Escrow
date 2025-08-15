import { BrowserRouter, Link, Route, Routes } from 'react-router-dom';
import Listings from './pages/Listings';
import MyListings from './pages/MyListings';
import { WalletContextProvider } from './lib/wallet';

export default function App() {
  return (
    <WalletContextProvider>
      <BrowserRouter>
        <nav style={{ display: 'flex', gap: '1rem' }}>
          <Link to="/">Listings</Link>
          <Link to="/my">My Listings</Link>
        </nav>
        <Routes>
          <Route path="/" element={<Listings />} />
          <Route path="/my" element={<MyListings />} />
        </Routes>
      </BrowserRouter>
    </WalletContextProvider>
  );
}
