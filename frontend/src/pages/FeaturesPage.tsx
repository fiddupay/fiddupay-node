import React from 'react'
import styles from './FeaturesPage.module.css'

const FeaturesPage: React.FC = () => {
  return (
    <div className={styles.featuresPage}>
      <div className={styles.container}>
        {/* Hero Section */}
        <section className={styles.hero}>
          <h1>Powerful Features for Modern Businesses</h1>
          <p>Everything you need to accept cryptocurrency payments with confidence and scale your business globally.</p>
        </section>

        {/* Main Features Grid */}
        <section className={styles.mainFeatures}>
          <div className={styles.featuresGrid}>
            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-bolt"></i>
              </div>
              <h3>Instant Processing</h3>
              <p>Real-time payment confirmation and automatic forwarding to your wallets within seconds of blockchain confirmation.</p>
              <ul>
                <li>Sub-second API responses</li>
                <li>Automatic wallet forwarding</li>
                <li>Real-time webhooks</li>
              </ul>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-network-wired"></i>
              </div>
              <h3>Multi-Network Support</h3>
              <p>Accept payments across 5 major blockchain networks with unified management and consistent experience.</p>
              <ul>
                <li>Solana, Ethereum, BSC</li>
                <li>Polygon, Arbitrum networks</li>
                <li>10 cryptocurrency options</li>
              </ul>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-shield-alt"></i>
              </div>
              <h3>Enterprise Security</h3>
              <p>Military-grade security with advanced encryption, multi-signature wallets, and real-time threat detection.</p>
              <ul>
                <li>AES-256 encryption</li>
                <li>Multi-signature wallets</li>
                <li>24/7 monitoring</li>
              </ul>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-code"></i>
              </div>
              <h3>Developer Friendly</h3>
              <p>Simple REST API with comprehensive documentation, SDKs, and tools for seamless integration.</p>
              <ul>
                <li>RESTful API design</li>
                <li>Multiple language SDKs</li>
                <li>Complete documentation</li>
              </ul>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-chart-line"></i>
              </div>
              <h3>Real-Time Analytics</h3>
              <p>Comprehensive dashboard with payment tracking, performance metrics, and business insights.</p>
              <ul>
                <li>Payment analytics</li>
                <li>Revenue tracking</li>
                <li>Performance metrics</li>
              </ul>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-mobile-alt"></i>
              </div>
              <h3>Mobile Optimized</h3>
              <p>QR code payments and mobile-first design ensure seamless experience across all devices.</p>
              <ul>
                <li>QR code generation</li>
                <li>Mobile-responsive UI</li>
                <li>Touch-friendly interface</li>
              </ul>
            </div>
          </div>
        </section>

        {/* Integration Section */}
        <section className={styles.integration}>
          <h2>Easy Integration</h2>
          <div className={styles.integrationContent}>
            <div className={styles.integrationText}>
              <h3>Get started in minutes</h3>
              <p>Our simple API and comprehensive SDKs make integration straightforward for developers of all skill levels.</p>
              <div className={styles.integrationFeatures}>
                <div className={styles.integrationFeature}>
                  <i className="fas fa-check"></i>
                  <span>5-minute setup</span>
                </div>
                <div className={styles.integrationFeature}>
                  <i className="fas fa-check"></i>
                  <span>No blockchain knowledge required</span>
                </div>
                <div className={styles.integrationFeature}>
                  <i className="fas fa-check"></i>
                  <span>Sandbox environment</span>
                </div>
              </div>
            </div>
            <div className={styles.codeExample}>
              <pre><code>{`// Create a payment
const payment = await fiddupay.payments.create({
  amount_usd: "100.00",
  crypto_type: "USDT_ETH",
  description: "Order #12345"
});

console.log(payment.payment_url);`}</code></pre>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}

export default FeaturesPage
