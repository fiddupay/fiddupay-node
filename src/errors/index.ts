// Custom error classes for FidduPay SDK

export class FidduPayError extends Error {
  public readonly type: string;
  public readonly code?: string;
  public readonly statusCode?: number;
  public readonly requestId?: string;

  constructor(message: string, type: string = 'fiddupay_error') {
    super(message);
    this.name = 'FidduPayError';
    this.type = type;
    Object.setPrototypeOf(this, FidduPayError.prototype);
  }
}

export class FidduPayAPIError extends FidduPayError {
  public readonly statusCode: number;
  public readonly code?: string;
  public readonly requestId?: string;

  constructor(
    message: string,
    statusCode: number,
    code?: string,
    requestId?: string
  ) {
    super(message, 'api_error');
    this.name = 'FidduPayAPIError';
    this.statusCode = statusCode;
    this.code = code;
    this.requestId = requestId;
    Object.setPrototypeOf(this, FidduPayAPIError.prototype);
  }
}

export class FidduPayValidationError extends FidduPayError {
  public readonly param?: string;

  constructor(message: string, param?: string) {
    super(message, 'validation_error');
    this.name = 'FidduPayValidationError';
    this.param = param;
    Object.setPrototypeOf(this, FidduPayValidationError.prototype);
  }
}

export class FidduPayAuthenticationError extends FidduPayError {
  constructor(message: string = 'Invalid API key provided') {
    super(message, 'authentication_error');
    this.name = 'FidduPayAuthenticationError';
    Object.setPrototypeOf(this, FidduPayAuthenticationError.prototype);
  }
}

export class FidduPayRateLimitError extends FidduPayError {
  public readonly retryAfter?: number;

  constructor(message: string = 'Too many requests', retryAfter?: number) {
    super(message, 'rate_limit_error');
    this.name = 'FidduPayRateLimitError';
    this.retryAfter = retryAfter;
    Object.setPrototypeOf(this, FidduPayRateLimitError.prototype);
  }
}

export class FidduPayConnectionError extends FidduPayError {
  constructor(message: string = 'Network connection failed') {
    super(message, 'connection_error');
    this.name = 'FidduPayConnectionError';
    Object.setPrototypeOf(this, FidduPayConnectionError.prototype);
  }
}
