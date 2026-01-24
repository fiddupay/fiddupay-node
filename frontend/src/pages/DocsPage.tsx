import React from 'react'
import styles from './DocsPage.module.css'

const DocsPage: React.FC = () => {
  return (
    <div className={styles.docsPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Documentation</h1>
          <p className={styles.subtitle}>
            Get started with PayFlow integration in minutes
          </p>
        </div>

        <div className={styles.content}>
          <div className={styles.quickStart}>
            <h2><i className="fas fa-rocket"></i> Quick Start</h2>
            <div className={styles.step}>
              <h3>1. Create Account</h3>
              <p>Sign up for a PayFlow account and verify your email address.</p>
            </div>
            <div className={styles.step}>
              <h3>2. Generate API Keys</h3>
              <p>Create API keys from your dashboard to authenticate requests.</p>
            </div>
            <div className={styles.step}>
              <h3>3. Create Payment</h3>
              <p>Use our API to create payment requests for your customers.</p>
            </div>
          </div>

          <div className={styles.apiExample}>
            <h2><i className="fas fa-code"></i> API Example</h2>
            <div className={styles.codeBlock}>
              <pre>
{`curl -X POST https://api.payflow.com/v1/payments \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "amount": "100.00",
    "currency": "USDT",
    "network": "ethereum",
    "callback_url": "https://yoursite.com/webhook"
  }'`}
              </pre>
            </div>
          </div>

          <div className={styles.resources}>
            <h2><i className="fas fa-book"></i> Resources</h2>
            <div className={styles.resourceGrid}>
              <a href="#" className={styles.resource}>
                <i className="fas fa-file-code"></i>
                <h3>API Reference</h3>
                <p>Complete API documentation with examples</p>
              </a>
              <a href="#" className={styles.resource}>
                <i className="fas fa-puzzle-piece"></i>
                <h3>SDKs</h3>
                <p>Official SDKs for popular programming languages</p>
              </a>
              <a href="#" className={styles.resource}>
                <i className="fas fa-question-circle"></i>
                <h3>FAQ</h3>
                <p>Frequently asked questions and troubleshooting</p>
              </a>
              <a href="#" className={styles.resource}>
                <i className="fas fa-comments"></i>
                <h3>Support</h3>
                <p>Get help from our technical support team</p>
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default DocsPage
