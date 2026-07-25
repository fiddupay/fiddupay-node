/**
 * Wallet Health & Auto-Provisioning — SDK Tests
 *
 * Covers:
 *  1. Method existence on client.customers
 *  2. Correct TypeScript signatures (argument count, return-type shape)
 *  3. URL construction (verified by mocking the underlying HTTP client)
 *  4. Correct mapping of API response fields (deposit_address → address, provisioned flag)
 *  5. Error path when auto-provision is not possible (network not enabled)
 */

import FidduPay from '../src';
import { FidduPayAPIError } from '../src/errors';

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

function buildClient() {
  return new FidduPay({ apiKey: 'sk_sandbox_wallet_health_tests' });
}

/** Patches the raw HTTP client inside a resource so we can intercept requests. */
function mockHttp(
  resource: any,
  method: 'get' | 'post',
  responseFactory: (path: string, body?: any) => any
) {
  const original = resource['client'][method].bind(resource['client']);
  resource['client'][method] = jest.fn((...args: any[]) => {
    return Promise.resolve(responseFactory(args[0], args[1]));
  });
  return () => { resource['client'][method] = original; };
}

// --------------------------------------------------------------------------
// Suite 1 — Method Availability
// --------------------------------------------------------------------------

describe('Wallet Health — Method Availability', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = buildClient();
  });

  it('should expose verifyAndRepairWallets on customers resource', () => {
    expect(client.customers.verifyAndRepairWallets).toBeDefined();
    expect(typeof client.customers.verifyAndRepairWallets).toBe('function');
  });

  it('should expose lookupAddress on customers resource', () => {
    expect(client.customers.lookupAddress).toBeDefined();
    expect(typeof client.customers.lookupAddress).toBe('function');
  });

  it('should expose auditWallets on customers resource', () => {
    expect(client.customers.auditWallets).toBeDefined();
    expect(typeof client.customers.auditWallets).toBeDefined();
    expect(typeof client.customers.auditWallets).toBe('function');
  });

  it('should expose getDepositAddress on customers resource', () => {
    expect(client.customers.getDepositAddress).toBeDefined();
    expect(typeof client.customers.getDepositAddress).toBe('function');
  });

  it('should have all new wallet-health methods alongside existing customer methods', () => {
    const requiredMethods = [
      'register',
      'list',
      'getWallets',
      'createWallets',
      'getBalances',
      'getDepositAddress',
      'getTransactions',
      'payMerchant',
      'updateStatus',
      'updatePermissions',
      'sweep',
      'deactivate',
      'getSummary',
      'bulkProvision',
      // new methods
      'verifyAndRepairWallets',
      'lookupAddress',
      'auditWallets',
    ];

    requiredMethods.forEach((method) => {
      expect(client.customers).toHaveProperty(method);
      expect(typeof (client.customers as any)[method]).toBe('function');
    });
  });
});

// --------------------------------------------------------------------------
// Suite 2 — getDepositAddress (updated return shape)
// --------------------------------------------------------------------------

describe('getDepositAddress — Updated Return Shape', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = buildClient();
  });

  it('should map deposit_address → address and expose provisioned = false for existing wallet', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      external_id: 'user_abc',
      crypto_type: 'USDT_BEP20',
      deposit_address: '0xabc123',
      provisioned: false,
    }));

    const result = await client.customers.getDepositAddress('user_abc', 'USDT_BEP20');

    expect(result.address).toBe('0xabc123');
    expect(result.crypto_type).toBe('USDT_BEP20');
    expect(result.external_id).toBe('user_abc');
    expect(result.provisioned).toBe(false);

    restore();
  });

  it('should map deposit_address → address and expose provisioned = true when auto-provisioned', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      external_id: 'new_user',
      crypto_type: 'ETH',
      deposit_address: '0xnewwallet999',
      provisioned: true,
    }));

    const result = await client.customers.getDepositAddress('new_user', 'ETH');

    expect(result.address).toBe('0xnewwallet999');
    expect(result.provisioned).toBe(true);

    restore();
  });

  it('should default provisioned to false if API omits the field (backward compat)', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      external_id: 'legacy_user',
      crypto_type: 'SOL',
      deposit_address: '5rBr6CFUA4Yi7uoX9JUgvC9PFzEjv5jNtu5NThZNEKqP',
      // provisioned omitted intentionally
    }));

    const result = await client.customers.getDepositAddress('legacy_user', 'SOL');

    expect(result.address).toBe('5rBr6CFUA4Yi7uoX9JUgvC9PFzEjv5jNtu5NThZNEKqP');
    expect(result.provisioned).toBe(false); // should default, not undefined
    restore();
  });

  it('should call correct endpoint path including external_id and crypto_type', async () => {
    const calledPaths: string[] = [];
    const restore = mockHttp(client.customers, 'get', (path) => {
      calledPaths.push(path);
      return {
        external_id: 'cust_xyz',
        crypto_type: 'BNB',
        deposit_address: '0xbnbaddr',
        provisioned: false,
      };
    });

    await client.customers.getDepositAddress('cust_xyz', 'BNB');

    expect(calledPaths).toHaveLength(1);
    expect(calledPaths[0]).toContain('cust_xyz');
    expect(calledPaths[0]).toContain('BNB');
    expect(calledPaths[0]).toContain('deposit-address');

    restore();
  });

  it('should support all EVM crypto types for deposit address retrieval', async () => {
    const evmTypes = ['ETH', 'USDT_ETH', 'BNB', 'USDT_BEP20', 'MATIC', 'USDT_POLYGON', 'ARB', 'USDT_ARBITRUM'];

    for (const crypto of evmTypes) {
      const restore = mockHttp(client.customers, 'get', () => ({
        external_id: 'evm_user',
        crypto_type: crypto,
        deposit_address: '0xsharedEvmAddress',
        provisioned: false,
      }));

      const result = await client.customers.getDepositAddress('evm_user', crypto);
      expect(result.address).toBe('0xsharedEvmAddress');
      expect(result.crypto_type).toBe(crypto);

      restore();
    }
  });

  it('should support SOL and USDT_SPL crypto types', async () => {
    const solTypes = ['SOL', 'USDT_SPL'];

    for (const crypto of solTypes) {
      const restore = mockHttp(client.customers, 'get', () => ({
        external_id: 'sol_user',
        crypto_type: crypto,
        deposit_address: 'SolanaAddr123',
        provisioned: false,
      }));

      const result = await client.customers.getDepositAddress('sol_user', crypto);
      expect(result.address).toBe('SolanaAddr123');

      restore();
    }
  });
});

// --------------------------------------------------------------------------
// Suite 3 — verifyAndRepairWallets
// --------------------------------------------------------------------------

describe('verifyAndRepairWallets', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = buildClient();
  });

  it('should call POST /api/v1/merchants/customers/verify-wallets', async () => {
    const calledPaths: string[] = [];
    const restore = mockHttp(client.customers, 'post', (path) => {
      calledPaths.push(path);
      return { status: 'completed', checked_customers: 55, repaired_wallets: 2 };
    });

    await client.customers.verifyAndRepairWallets();

    expect(calledPaths).toHaveLength(1);
    expect(calledPaths[0]).toBe('/api/v1/merchants/customers/verify-wallets');

    restore();
  });

  it('should return correct shape: { status, checked_customers, repaired_wallets }', async () => {
    const restore = mockHttp(client.customers, 'post', () => ({
      status: 'completed',
      checked_customers: 177,
      repaired_wallets: 3,
    }));

    const result = await client.customers.verifyAndRepairWallets();

    expect(result).toHaveProperty('status');
    expect(result).toHaveProperty('checked_customers');
    expect(result).toHaveProperty('repaired_wallets');
    expect(result.checked_customers).toBe(177);
    expect(result.repaired_wallets).toBe(3);
    expect(result.status).toBe('completed');

    restore();
  });

  it('should report zero repaired_wallets when all wallets are already in order', async () => {
    const restore = mockHttp(client.customers, 'post', () => ({
      status: 'completed',
      checked_customers: 42,
      repaired_wallets: 0,
    }));

    const result = await client.customers.verifyAndRepairWallets();
    expect(result.repaired_wallets).toBe(0);
    expect(result.checked_customers).toBe(42);

    restore();
  });

  it('should report repaired_wallets > 0 when missing wallets are found and provisioned', async () => {
    const restore = mockHttp(client.customers, 'post', () => ({
      status: 'completed',
      checked_customers: 10,
      repaired_wallets: 4,
    }));

    const result = await client.customers.verifyAndRepairWallets();
    expect(result.repaired_wallets).toBeGreaterThan(0);

    restore();
  });
});

// --------------------------------------------------------------------------
// Suite 4 — lookupAddress
// --------------------------------------------------------------------------

describe('lookupAddress', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = buildClient();
  });

  it('should call GET /api/v1/merchants/customers/lookup-address/:address', async () => {
    const calledPaths: string[] = [];
    const restore = mockHttp(client.customers, 'get', (path) => {
      calledPaths.push(path);
      return { found: true, status: 'ACTIVE', customer: {}, wallet: {} };
    });

    await client.customers.lookupAddress('0xTestAddress123');

    expect(calledPaths.some((p) => p.includes('lookup-address'))).toBe(true);
    expect(calledPaths.some((p) => p.includes('0xTestAddress123'))).toBe(true);

    restore();
  });

  it('should return found=true with ACTIVE status for current wallet', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      found: true,
      status: 'ACTIVE',
      customer: {
        id: 42,
        external_id: 'user_active',
        email: 'active@example.com',
      },
      wallet: {
        address: '0xActiveAddress',
        crypto_type: 'ETH',
        network: 'ETHEREUM',
        sandbox_mode: false,
        created_at: '2025-01-01T00:00:00Z',
      },
    }));

    const result = await client.customers.lookupAddress('0xActiveAddress');

    expect(result.found).toBe(true);
    expect(result.status).toBe('ACTIVE');
    expect(result.customer.external_id).toBe('user_active');
    expect(result.wallet.network).toBe('ETHEREUM');

    restore();
  });

  it('should return found=true with HISTORICAL status for old archived wallet', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      found: true,
      status: 'HISTORICAL',
      customer: {
        id: 7,
        external_id: 'old_user',
        email: 'old@example.com',
      },
      wallet: {
        address: '0xOldAddress',
        crypto_type: 'USDT_BEP20',
        network: 'ETHEREUM',
        sandbox_mode: false,
        reason: 'Customer wallet re-provisioned',
        created_at: '2024-06-15T00:00:00Z',
      },
    }));

    const result = await client.customers.lookupAddress('0xOldAddress');

    expect(result.found).toBe(true);
    expect(result.status).toBe('HISTORICAL');
    expect(result.wallet.reason).toBe('Customer wallet re-provisioned');

    restore();
  });

  it('should return found=false for an address not belonging to any customer', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({ found: false }));

    const result = await client.customers.lookupAddress('0xUnknownAddress');

    expect(result.found).toBe(false);

    restore();
  });

  it('should handle Solana address lookup', async () => {
    const solAddress = '5rBr6CFUA4Yi7uoX9JUgvC9PFzEjv5jNtu5NThZNEKqP';

    const restore = mockHttp(client.customers, 'get', () => ({
      found: true,
      status: 'ACTIVE',
      customer: { id: 99, external_id: 'sol_user', email: 'sol@example.com' },
      wallet: { address: solAddress, crypto_type: 'SOL', network: 'SOLANA', sandbox_mode: false, created_at: '2025-01-01T00:00:00Z' },
    }));

    const result = await client.customers.lookupAddress(solAddress);

    expect(result.found).toBe(true);
    expect(result.wallet.network).toBe('SOLANA');

    restore();
  });
});

// --------------------------------------------------------------------------
// Suite 5 — auditWallets
// --------------------------------------------------------------------------

describe('auditWallets', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = buildClient();
  });

  it('should call GET /api/v1/merchants/customers/wallets-audit', async () => {
    const calledPaths: string[] = [];
    const restore = mockHttp(client.customers, 'get', (path) => {
      calledPaths.push(path);
      return { active: [], historical: [] };
    });

    await client.customers.auditWallets();

    expect(calledPaths.some((p) => p.includes('wallets-audit'))).toBe(true);

    restore();
  });

  it('should return shape { active: [], historical: [] } for empty merchant', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({ active: [], historical: [] }));

    const result = await client.customers.auditWallets();

    expect(result).toHaveProperty('active');
    expect(result).toHaveProperty('historical');
    expect(Array.isArray(result.active)).toBe(true);
    expect(Array.isArray(result.historical)).toBe(true);

    restore();
  });

  it('should return active wallet records with expected fields', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      active: [
        {
          external_id: 'user_1',
          email: 'user1@example.com',
          address: '0xActiveAddr',
          crypto_type: 'ETH',
          network: 'ETHEREUM',
          sandbox_mode: false,
          status: 'ACTIVE',
          created_at: '2025-01-01T00:00:00Z',
        },
      ],
      historical: [],
    }));

    const result = await client.customers.auditWallets();

    expect(result.active).toHaveLength(1);
    expect(result.active[0].external_id).toBe('user_1');
    expect(result.active[0].status).toBe('ACTIVE');
    expect(result.active[0].network).toBe('ETHEREUM');

    restore();
  });

  it('should return historical wallet records with expected fields', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      active: [],
      historical: [
        {
          external_id: 'user_2',
          email: 'user2@example.com',
          address: '0xOldAddr',
          crypto_type: 'USDT_BEP20',
          network: 'ETHEREUM',
          sandbox_mode: false,
          status: 'HISTORICAL',
          reason: 'Customer wallet re-provisioned',
          created_at: '2024-03-10T00:00:00Z',
        },
      ],
    }));

    const result = await client.customers.auditWallets();

    expect(result.historical).toHaveLength(1);
    expect(result.historical[0].status).toBe('HISTORICAL');
    expect(result.historical[0].reason).toBe('Customer wallet re-provisioned');

    restore();
  });

  it('should correctly separate active and historical wallets', async () => {
    const restore = mockHttp(client.customers, 'get', () => ({
      active: [
        { external_id: 'u1', email: 'u1@e.com', address: '0xNew', crypto_type: 'ETH', network: 'ETHEREUM', sandbox_mode: false, status: 'ACTIVE', created_at: '2025-01-01T00:00:00Z' },
      ],
      historical: [
        { external_id: 'u1', email: 'u1@e.com', address: '0xOld', crypto_type: 'ETH', network: 'ETHEREUM', sandbox_mode: false, status: 'HISTORICAL', reason: 'Reprovisioned', created_at: '2024-01-01T00:00:00Z' },
      ],
    }));

    const result = await client.customers.auditWallets();

    expect(result.active.every((w: any) => w.status === 'ACTIVE')).toBe(true);
    expect(result.historical.every((w: any) => w.status === 'HISTORICAL')).toBe(true);

    restore();
  });
});

// --------------------------------------------------------------------------
// Suite 6 — Error Handling
// --------------------------------------------------------------------------

describe('Wallet Health — Error Handling', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = buildClient();
  });

  it('lookupAddress: should propagate API errors correctly', async () => {
    (client.customers as any)['client']['get'] = jest.fn(() =>
      Promise.reject(new FidduPayAPIError('Not found', 404, 'NOT_FOUND'))
    );

    await expect(client.customers.lookupAddress('0xBadAddr')).rejects.toThrow(FidduPayAPIError);
  });

  it('verifyAndRepairWallets: should propagate server errors', async () => {
    (client.customers as any)['client']['post'] = jest.fn(() =>
      Promise.reject(new FidduPayAPIError('Internal server error', 500, 'INTERNAL_ERROR'))
    );

    await expect(client.customers.verifyAndRepairWallets()).rejects.toThrow(FidduPayAPIError);
  });

  it('getDepositAddress: should propagate errors if network not enabled for merchant', async () => {
    (client.customers as any)['client']['get'] = jest.fn(() =>
      Promise.reject(
        new FidduPayAPIError(
          'Could not provision wallet for BTC — network may not be enabled for this merchant',
          400,
          'VALIDATION_ERROR'
        )
      )
    );

    await expect(
      client.customers.getDepositAddress('cust_123', 'BTC')
    ).rejects.toThrow('network may not be enabled for this merchant');
  });
});
