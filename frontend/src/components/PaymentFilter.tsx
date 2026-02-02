import React, { useState } from 'react';

interface PaymentFilters {
  status?: string;
  crypto_type?: string;
  blockchain?: string;
  start_date?: string;
  end_date?: string;
  min_amount?: number;
  max_amount?: number;
}

interface PaymentFilterProps {
  onFiltersChange: (filters: PaymentFilters) => void;
  onClear: () => void;
}

export const PaymentFilter: React.FC<PaymentFilterProps> = ({ onFiltersChange, onClear }) => {
  const [filters, setFilters] = useState<PaymentFilters>({});

  const handleFilterChange = (key: keyof PaymentFilters, value: string | number) => {
    const newFilters = { ...filters, [key]: value || undefined };
    setFilters(newFilters);
    onFiltersChange(newFilters);
  };

  const handleClear = () => {
    setFilters({});
    onClear();
  };

  return (
    <div className="bg-white p-4 rounded-lg shadow mb-6">
      <h3 className="text-lg font-semibold mb-4">Filter Payments</h3>
      
      <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 gap-4">
        <div>
          <label className="block text-sm font-medium mb-1">Status</label>
          <select
            value={filters.status || ''}
            onChange={(e) => handleFilterChange('status', e.target.value)}
            className="w-full p-2 border rounded-md"
          >
            <option value="">All Statuses</option>
            <option value="pending">Pending</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="cancelled">Cancelled</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Cryptocurrency</label>
          <select
            value={filters.crypto_type || ''}
            onChange={(e) => handleFilterChange('crypto_type', e.target.value)}
            className="w-full p-2 border rounded-md"
          >
            <option value="">All Cryptocurrencies</option>
            <option value="SOL">Solana (SOL)</option>
            <option value="ETH">Ethereum (ETH)</option>
            <option value="BNB">Binance Coin (BNB)</option>
            <option value="MATIC">Polygon (MATIC)</option>
            <option value="ARB">Arbitrum (ARB)</option>
            <option value="USDT">USDT</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Blockchain</label>
          <select
            value={filters.blockchain || ''}
            onChange={(e) => handleFilterChange('blockchain', e.target.value)}
            className="w-full p-2 border rounded-md"
          >
            <option value="">All Blockchains</option>
            <option value="solana">Solana</option>
            <option value="ethereum">Ethereum</option>
            <option value="bsc">Binance Smart Chain</option>
            <option value="polygon">Polygon</option>
            <option value="arbitrum">Arbitrum</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Start Date</label>
          <input
            type="date"
            value={filters.start_date || ''}
            onChange={(e) => handleFilterChange('start_date', e.target.value)}
            className="w-full p-2 border rounded-md"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">End Date</label>
          <input
            type="date"
            value={filters.end_date || ''}
            onChange={(e) => handleFilterChange('end_date', e.target.value)}
            className="w-full p-2 border rounded-md"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Min Amount ($)</label>
          <input
            type="number"
            step="0.01"
            value={filters.min_amount || ''}
            onChange={(e) => handleFilterChange('min_amount', parseFloat(e.target.value))}
            className="w-full p-2 border rounded-md"
            placeholder="0.00"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Max Amount ($)</label>
          <input
            type="number"
            step="0.01"
            value={filters.max_amount || ''}
            onChange={(e) => handleFilterChange('max_amount', parseFloat(e.target.value))}
            className="w-full p-2 border rounded-md"
            placeholder="1000.00"
          />
        </div>

        <div className="flex items-end">
          <button
            onClick={handleClear}
            className="w-full px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600"
          >
            Clear Filters
          </button>
        </div>
      </div>
    </div>
  );
};
