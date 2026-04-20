import React from 'react'
import styles from '@/styles/pages/FeaturesPage.module.css'

const FeaturesPage: React.FC = () => {
  return (
    <div className={styles.featuresPage}>
      {/* Background Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        {/* Hero Section */}
        <section className={`${styles.hero} animate-fade-in-up`}>
          <h1>Powerful Features for <span className={styles.gradientText}>Modern Businesses</span></h1>
          <p>Everything you need to accept cryptocurrency payments with confidence and scale your business globally.</p>
        </section>

        {/* Main Features Grid */}
        <section className={styles.mainFeatures}>
          <div className={styles.featuresGrid}>
            <div className={`${styles.featureCard} animate-fade-in-up`} style={{ animationDelay: '0.1s' }}>
              <div className={styles.featureWatermark}><i className="fas fa-bolt"></i></div>
              <div className={`${styles.featureIcon} ${styles.iconPrimary}`}>
                <i className="fas fa-bolt"></i>
              </div>
              <h3>Instant Processing</h3>
              <p>Real-time payment confirmation and automatic forwarding to your wallets within seconds of blockchain confirmation.</p>
              <ul>
                <li><i className="fas fa-check text-secondary"></i> Sub-second API responses</li>
                <li><i className="fas fa-check text-secondary"></i> Automatic wallet forwarding</li>
                <li><i className="fas fa-check text-secondary"></i> Real-time webhooks</li>
              </ul>
            </div>

            <div className={`${styles.featureCard} animate-fade-in-up`} style={{ animationDelay: '0.2s' }}>
              <div className={styles.featureWatermark}><i className="fas fa-network-wired"></i></div>
              <div className={`${styles.featureIcon} ${styles.iconSecondary}`}>
                <i className="fas fa-network-wired"></i>
              </div>
              <h3>Multi-Network Support</h3>
              <p>Accept payments across 6 major blockchain networks with unified management and consistent experience.</p>
              <ul>
                <li><i className="fas fa-check text-secondary"></i> Bitcoin, Solana, Ethereum, BSC</li>
                <li><i className="fas fa-check text-secondary"></i> Polygon, Arbitrum networks</li>
                <li><i className="fas fa-check text-secondary"></i> 12 cryptocurrency options</li>
              </ul>
            </div>

            <div className={`${styles.featureCard} animate-fade-in-up`} style={{ animationDelay: '0.3s' }}>
              <div className={styles.featureWatermark}><i className="fas fa-shield-alt"></i></div>
              <div className={`${styles.featureIcon} ${styles.iconPrimary}`}>
                <i className="fas fa-shield-alt"></i>
              </div>
              <h3>Enterprise Security</h3>
              <p>Military-grade security with advanced encryption, multi-signature wallets, and real-time threat detection.</p>
              <ul>
                <li><i className="fas fa-check text-secondary"></i> AES-256 encryption</li>
                <li><i className="fas fa-check text-secondary"></i> Multi-signature wallets</li>
                <li><i className="fas fa-check text-secondary"></i> 24/7 monitoring</li>
              </ul>
            </div>

            <div className={`${styles.featureCard} animate-fade-in-up`} style={{ animationDelay: '0.4s' }}>
              <div className={styles.featureWatermark}><i className="fas fa-code"></i></div>
              <div className={`${styles.featureIcon} ${styles.iconSecondary}`}>
                <i className="fas fa-code"></i>
              </div>
              <h3>Developer Friendly</h3>
              <p>Simple REST API with comprehensive documentation, SDKs, and tools for seamless integration.</p>
              <ul>
                <li><i className="fas fa-check text-secondary"></i> RESTful API design</li>
                <li><i className="fas fa-check text-secondary"></i> Multiple language SDKs</li>
                <li><i className="fas fa-check text-secondary"></i> Complete documentation</li>
              </ul>
            </div>

            <div className={`${styles.featureCard} animate-fade-in-up`} style={{ animationDelay: '0.5s' }}>
              <div className={styles.featureWatermark}><i className="fas fa-chart-line"></i></div>
              <div className={`${styles.featureIcon} ${styles.iconPrimary}`}>
                <i className="fas fa-chart-line"></i>
              </div>
              <h3>Real-Time Analytics</h3>
              <p>Comprehensive dashboard with payment tracking, performance metrics, and business insights.</p>
              <ul>
                <li><i className="fas fa-check text-secondary"></i> Payment analytics</li>
                <li><i className="fas fa-check text-secondary"></i> Revenue tracking</li>
                <li><i className="fas fa-check text-secondary"></i> Performance metrics</li>
              </ul>
            </div>

            <div className={`${styles.featureCard} animate-fade-in-up`} style={{ animationDelay: '0.6s' }}>
              <div className={styles.featureWatermark}><i className="fas fa-mobile-alt"></i></div>
              <div className={`${styles.featureIcon} ${styles.iconSecondary}`}>
                <i className="fas fa-mobile-alt"></i>
              </div>
              <h3>Mobile Optimized</h3>
              <p>QR code payments and mobile-first design ensure seamless experience across all devices.</p>
              <ul>
                <li><i className="fas fa-check text-secondary"></i> QR code generation</li>
                <li><i className="fas fa-check text-secondary"></i> Mobile-responsive UI</li>
                <li><i className="fas fa-check text-secondary"></i> Touch-friendly interface</li>
              </ul>
            </div>
          </div>
        </section>

        {/* Integration Section */}
        <section className={styles.integration}>
          <div className={styles.integrationContent}>
            <div className={styles.integrationText}>
              <h2>Easy Integration</h2>
              <p className={styles.integrationSubtitle}>Get started in minutes with our powerful SDKs.</p>
              <div className={styles.integrationFeaturesList}>
                <div className={styles.integrationFeatureItem}>
                  <i className="fas fa-check-circle"></i> Complete Sandbox Environment
                </div>
                <div className={styles.integrationFeatureItem}>
                  <i className="fas fa-check-circle"></i> Type-safe Typescript Support
                </div>
                <div className={styles.integrationFeatureItem}>
                  <i className="fas fa-check-circle"></i> Webhook Signatures Included
                </div>
              </div>
            </div>
            
            <div className={styles.codeExampleContainer}>
              <div className={styles.codeHeader}>
                <div className={styles.macButtons}>
                  <span></span><span></span><span></span>
                </div>
                <div className={styles.codeTitle}>payment.ts</div>
              </div>
              <pre className={styles.codeExample}><code>{`import { FidduPay } from 'fiddupay-sdk';

const client = new FidduPay({ apiKey: 'sk_test_...' });

// Create an instant checkout session
const checkout = await client.payments.create({
  amount_usd: 125.00,
  crypto_type: "USDC_SOL",
  description: "Premium Plan Upgrade",
  metadata: { userId: "user_88291" }
});

console.log(checkout.payment_url);`}</code></pre>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}

export default FeaturesPage
