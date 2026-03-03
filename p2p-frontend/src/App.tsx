import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Marketplace from './pages/Marketplace';
import TradingRoom from './pages/TradingRoom';
import WalletDashboard from './pages/WalletDashboard';
import { Layout } from './components/Layout';

function App() {
  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<Marketplace />} />
          <Route path="/orders/:tradeId" element={<TradingRoom />} />
          <Route path="/wallet" element={<WalletDashboard />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}

export default App;
