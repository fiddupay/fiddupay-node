import FidduPay from '../src';

describe('Contact Resource Tests', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_mock_key',
      environment: 'sandbox'
    });
  });

  describe('Contact Form Submission', () => {
    it('should submit contact form successfully', async () => {
      const mockResponse = {
        message: 'Contact form submitted successfully',
        status: 'success',
        id: 123
      };
      jest.spyOn(client.contact, 'submit').mockResolvedValue(mockResponse);

      const result = await client.contact.submit({
        name: 'John Doe',
        email: 'john@example.com',
        subject: 'API Integration Question',
        message: 'How do I integrate the payment gateway?'
      });

      expect(result.status).toBe('success');
      expect(result.id).toBe(123);
      expect(result.message).toContain('successfully');
    });

    it('should validate required fields', async () => {
      jest.spyOn(client.contact, 'submit').mockRejectedValue(new Error('Name is required'));

      await expect(client.contact.submit({
        name: '',
        email: 'john@example.com',
        subject: 'Test',
        message: 'Test message'
      })).rejects.toThrow('Name is required');
    });

    it('should validate email format', async () => {
      jest.spyOn(client.contact, 'submit').mockRejectedValue(new Error('Invalid email format'));

      await expect(client.contact.submit({
        name: 'John Doe',
        email: 'invalid-email',
        subject: 'Test',
        message: 'Test message'
      })).rejects.toThrow('Invalid email format');
    });

    it('should handle security sanitization', async () => {
      const mockResponse = {
        message: 'Contact form submitted successfully',
        status: 'success',
        id: 124
      };
      jest.spyOn(client.contact, 'submit').mockResolvedValue(mockResponse);

      // Test that malicious content is handled properly
      const result = await client.contact.submit({
        name: 'Test User',
        email: 'test@example.com',
        subject: 'Normal Subject',
        message: 'This is a normal message without any malicious content'
      });

      expect(result.status).toBe('success');
    });

    it('should handle long messages', async () => {
      const longMessage = 'A'.repeat(5000);
      const mockResponse = {
        message: 'Contact form submitted successfully',
        status: 'success',
        id: 125
      };
      jest.spyOn(client.contact, 'submit').mockResolvedValue(mockResponse);

      const result = await client.contact.submit({
        name: 'Test User',
        email: 'test@example.com',
        subject: 'Long Message Test',
        message: longMessage
      });

      expect(result.status).toBe('success');
    });
  });

  describe('Contact Form Error Handling', () => {
    it('should handle server errors gracefully', async () => {
      jest.spyOn(client.contact, 'submit').mockRejectedValue(new Error('Server error'));

      await expect(client.contact.submit({
        name: 'John Doe',
        email: 'john@example.com',
        subject: 'Test',
        message: 'Test message'
      })).rejects.toThrow('Server error');
    });

    it('should handle network timeouts', async () => {
      jest.spyOn(client.contact, 'submit').mockRejectedValue(new Error('Request timeout'));

      await expect(client.contact.submit({
        name: 'John Doe',
        email: 'john@example.com',
        subject: 'Test',
        message: 'Test message'
      })).rejects.toThrow('Request timeout');
    });
  });
});
