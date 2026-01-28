const axios = require('axios');

async function testAcknowledgeAlert() {
  try {
    // Get admin token
    const loginResponse = await axios.post('http://127.0.0.1:8080/api/v1/merchant/login', {
      email: 'superadmin@fiddupay.com',
      password: 'dummy'
    });
    
    const adminToken = loginResponse.data.api_key;
    console.log('✅ Got admin token:', adminToken.substring(0, 20) + '...');
    
    // Test the acknowledge alert endpoint with very short timeout
    console.log('🔄 Testing acknowledge alert endpoint...');
    
    const response = await axios.post(
      'http://127.0.0.1:8080/api/v1/admin/security/alerts/test_alert/acknowledge',
      {},
      {
        headers: { 'Authorization': `Bearer ${adminToken}` },
        timeout: 2000 // 2 second timeout
      }
    );
    
    console.log('✅ Success:', response.status, response.data);
    
  } catch (error) {
    if (error.code === 'ECONNABORTED') {
      console.log('❌ Timeout - endpoint is hanging');
    } else if (error.response) {
      console.log('❌ HTTP Error:', error.response.status, error.response.data);
    } else {
      console.log('❌ Network Error:', error.message);
    }
  }
}

testAcknowledgeAlert();
