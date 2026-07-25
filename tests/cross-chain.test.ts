// Delora Cross-Chain Endpoint Tests

import FidduPay from '../src';

describe('FidduPay SDK - CrossChain Resource', () => {
  let client: FidduPay;

  beforeAll(() => {
    client = new FidduPay({
      apiKey: 'sk_sandbox_cross_chain_test',
      timeout: 30000,
    });
  });

  describe('Resource Initialization', () => {
    it('should expose crossChain resource', () => {
      expect(client.crossChain).toBeDefined();
    });

    it('should have all cross-chain methods', () => {
      expect(typeof client.crossChain.getQuote).toBe('function');
      expect(typeof client.crossChain.registerTx).toBe('function');
      expect(typeof client.crossChain.getStatus).toBe('function');
      expect(typeof client.crossChain.getSupportedChains).toBe('function');
      expect(typeof client.crossChain.getTokensForChain).toBe('function');
    });
  });

  describe('getQuote() validation', () => {
    it('should reject empty link_id', async () => {
      await expect(client.crossChain.getQuote({
        link_id: '',
        sender_address: '0x1234567890123456789012345678901234567890',
        origin_chain_id: 137,
        origin_currency: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
      })).rejects.toThrow('link_id is required');
    });

    it('should reject empty sender_address', async () => {
      await expect(client.crossChain.getQuote({
        link_id: 'pay_abc',
        sender_address: '',
        origin_chain_id: 137,
        origin_currency: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
      })).rejects.toThrow('sender_address is required');
    });

    it('should reject zero origin_chain_id', async () => {
      await expect(client.crossChain.getQuote({
        link_id: 'pay_abc',
        sender_address: '0x1234567890123456789012345678901234567890',
        origin_chain_id: 0,
        origin_currency: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
      })).rejects.toThrow('Valid origin_chain_id is required');
    });

    it('should reject empty origin_currency', async () => {
      await expect(client.crossChain.getQuote({
        link_id: 'pay_abc',
        sender_address: '0x1234567890123456789012345678901234567890',
        origin_chain_id: 137,
        origin_currency: '',
      })).rejects.toThrow('origin_currency is required');
    });

    it('should reject invalid EVM address (too short)', async () => {
      await expect(client.crossChain.getQuote({
        link_id: 'pay_abc',
        sender_address: '0x123',
        origin_chain_id: 137,
        origin_currency: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
      })).rejects.toThrow('Invalid EVM sender_address');
    });

    it('should accept valid 0x-prefixed addresses', async () => {
      // This will still fail at the HTTP level (no real server),
      // but the validation should pass — the error will be a connection error
      try {
        await client.crossChain.getQuote({
          link_id: 'pay_abc',
          sender_address: '0x1234567890123456789012345678901234567890',
          origin_chain_id: 137,
          origin_currency: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
        });
      } catch (e: any) {
        // Validation passed, error is from HTTP (expected since no server)
        expect(e.message).not.toContain('link_id');
        expect(e.message).not.toContain('sender_address');
        expect(e.message).not.toContain('origin_chain_id');
      }
    });

    it('should accept valid Solana base58 addresses', async () => {
      try {
        await client.crossChain.getQuote({
          link_id: 'pay_abc',
          sender_address: 'DRpbCBMxVnDK7maPMoGQFix5grYex4Sm5CwFLCoL29sA',
          origin_chain_id: 137,
          origin_currency: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
        });
      } catch (e: any) {
        expect(e.message).not.toContain('sender_address');
      }
    });
  });

  describe('registerTx() validation', () => {
    it('should reject empty quote_id', async () => {
      await expect(client.crossChain.registerTx({
        quote_id: '',
        tx_hash: '0xabc',
        sender_address: '0x1234567890123456789012345678901234567890',
      })).rejects.toThrow('quote_id is required');
    });

    it('should reject empty tx_hash', async () => {
      await expect(client.crossChain.registerTx({
        quote_id: '550e8400-e29b-41d4-a716-446655440000',
        tx_hash: '',
        sender_address: '0x1234567890123456789012345678901234567890',
      })).rejects.toThrow('tx_hash is required');
    });

    it('should reject empty sender_address', async () => {
      await expect(client.crossChain.registerTx({
        quote_id: '550e8400-e29b-41d4-a716-446655440000',
        tx_hash: '0xabc',
        sender_address: '',
      })).rejects.toThrow('sender_address is required');
    });
  });

  describe('getStatus() validation', () => {
    it('should reject empty link_id', async () => {
      await expect(client.crossChain.getStatus('')).rejects.toThrow('link_id is required');
    });
  });

  describe('getTokensForChain() validation', () => {
    it('should reject zero chain_id', async () => {
      await expect(client.crossChain.getTokensForChain(0)).rejects.toThrow('Valid chain_id is required');
    });

    it('should reject negative chain_id', async () => {
      await expect(client.crossChain.getTokensForChain(-1)).rejects.toThrow('Valid chain_id is required');
    });
  });
});
