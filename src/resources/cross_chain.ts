import { HttpClient } from '../client';
import {
  CrossChainQuoteRequest,
  CrossChainQuoteResponse,
  CrossChainRegisterRequest,
  CrossChainStatusResponse,
  ChainSummary,
  TokenSummary,
} from '../types';
import { FidduPayValidationError } from '../errors';

export class CrossChain {
  constructor(private client: HttpClient) {}

  /**
   * Get a cross-chain swap quote for paying a merchant invoice with any token/chain.
   *
   * @example
   * ```ts
   * const quote = await client.crossChain.getQuote({
   *   link_id: "pay_abc123",
   *   sender_address: "0x...",
   *   origin_chain_id: 137,
   *   origin_currency: "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"
   * });
   * // Use quote.calldata for wallet signing
   * ```
   */
  async getQuote(params: CrossChainQuoteRequest): Promise<CrossChainQuoteResponse> {
    this.validateQuoteParams(params);

    const query = new URLSearchParams({
      link_id: params.link_id,
      sender_address: params.sender_address,
      origin_chain_id: params.origin_chain_id.toString(),
      origin_currency: params.origin_currency,
    });

    return this.client.get<CrossChainQuoteResponse>(
      `/api/v1/payments/cross-chain-quote?${query.toString()}`
    );
  }

  /**
   * Register a transaction hash after the customer signs and broadcasts
   * the cross-chain swap.
   *
   * @example
   * ```ts
   * const status = await client.crossChain.registerTx({
   *   quote_id: "550e8400-e29b-41d4-a716-446655440000",
   *   tx_hash: "0x...",
   *   sender_address: "0x..."
   * });
   * ```
   */
  async registerTx(params: CrossChainRegisterRequest): Promise<CrossChainStatusResponse> {
    if (!params.quote_id) {
      throw new FidduPayValidationError('quote_id is required', 'quote_id');
    }
    if (!params.tx_hash || params.tx_hash.trim().length === 0) {
      throw new FidduPayValidationError('tx_hash is required', 'tx_hash');
    }
    if (!params.sender_address || params.sender_address.trim().length === 0) {
      throw new FidduPayValidationError('sender_address is required', 'sender_address');
    }

    return this.client.post<CrossChainStatusResponse>(
      '/api/v1/payments/cross-chain-register',
      params
    );
  }

  /**
   * Get the real-time status of a cross-chain payment for a given payment link.
   *
   * @example
   * ```ts
   * const status = await client.crossChain.getStatus("pay_abc123");
   * if (status.status === "completed") {
   *   // Payment complete
   * }
   * ```
   */
  async getStatus(linkId: string): Promise<CrossChainStatusResponse> {
    if (!linkId) {
      throw new FidduPayValidationError('link_id is required', 'link_id');
    }

    return this.client.get<CrossChainStatusResponse>(
      `/api/v1/payments/cross-chain-status/${encodeURIComponent(linkId)}`
    );
  }

  /**
   * Get all supported chains for cross-chain payments.
   *
   * @example
   * ```ts
   * const chains = await client.crossChain.getSupportedChains();
   * // Render chain picker in UI
   * ```
   */
  async getSupportedChains(): Promise<ChainSummary[]> {
    return this.client.get<ChainSummary[]>('/api/v1/payments/cross-chain/chains');
  }

  /**
   * Get all supported tokens for a specific chain.
   *
   * @param chainId - Chain ID (e.g., 1=Ethereum, 137=Polygon, 42161=Arbitrum)
   *
   * @example
   * ```ts
   * const tokens = await client.crossChain.getTokensForChain(137);
   * // tokens contains all supported ERC-20 tokens on Polygon
   * ```
   */
  async getTokensForChain(chainId: number): Promise<TokenSummary[]> {
    if (!chainId || chainId <= 0) {
      throw new FidduPayValidationError('Valid chain_id is required', 'chain_id');
    }

    return this.client.get<TokenSummary[]>(
      `/api/v1/payments/cross-chain/tokens/${chainId}`
    );
  }

  private validateQuoteParams(params: CrossChainQuoteRequest): void {
    if (!params.link_id) {
      throw new FidduPayValidationError('link_id is required', 'link_id');
    }
    if (!params.sender_address || params.sender_address.trim().length === 0) {
      throw new FidduPayValidationError('sender_address is required', 'sender_address');
    }
    if (!params.origin_chain_id || params.origin_chain_id <= 0) {
      throw new FidduPayValidationError('Valid origin_chain_id is required', 'origin_chain_id');
    }
    if (!params.origin_currency || params.origin_currency.trim().length === 0) {
      throw new FidduPayValidationError('origin_currency is required', 'origin_currency');
    }

    // Validate EVM address format (0x + 40 hex chars) or Solana base58
    const addr = params.sender_address.trim();
    if (addr.startsWith('0x')) {
      if (addr.length !== 42) {
        throw new FidduPayValidationError(
          'Invalid EVM sender_address format (expected 0x + 40 hex chars)',
          'sender_address'
        );
      }
    } else if (addr.length < 32 || addr.length > 44) {
      throw new FidduPayValidationError(
        'Invalid sender_address format',
        'sender_address'
      );
    }
  }
}
