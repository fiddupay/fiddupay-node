import React from 'react'
import { MdShield, MdHistory, MdVerified, MdFingerprint } from 'react-icons/md'
import styles from '@/styles/pages/PublicSecurityPage.module.css'

const PublicSecurityPage: React.FC = () => {
    return (
        <div className={styles.container}>
            {/* Hero Section */}
            <section className={styles.heroSection}>
                <div className={styles.heroContent}>
                    <div className={styles.badge}>Security & Trust</div>
                    <h1 className={styles.title}>Secure, Transparent, Future-Ready</h1>
                    <p className={styles.subtitle}>
                        FidduPay is entering its Beta launch with a security-first architecture. 
                        We combine multi-layered encryption with a robust multi-chain foundation 
                        to safeguard every transaction from day one.
                    </p>
                    <div className={styles.heroActions}>
                        <button className={styles.primaryBtn}>View Security Features</button>
                        <button className={styles.secondaryBtn}>Developer Docs</button>
                    </div>
                </div>
                <div className={styles.heroImageWrapper}>
                    <img 
                        src="/security_trust_marketing_page_hero_1774967148421.png" 
                        alt="Security Infrastructure Visualization" 
                        className={styles.heroImage}
                    />
                </div>
            </section>

            {/* Core Pillars */}
            <section className={styles.pillarsSection}>
                <h2 className={styles.sectionTitle}>Built for Scale & Security</h2>
                <div className={styles.pillarsGrid}>
                    <div className={styles.pillarCard}>
                        <MdShield className={styles.pillarIcon} />
                        <h3>Encrypted by Default</h3>
                        <p>Merchant data and sensitive information are protected by AES-256 encryption at rest and TLS 1.3 in transit.</p>
                    </div>
                    <div className={styles.pillarCard}>
                        <MdFingerprint className={styles.pillarIcon} />
                        <h3>Identity Protection</h3>
                        <p>Our dashboard supports secure authentication methods to prevent unauthorized account access.</p>
                    </div>
                    <div className={styles.pillarCard}>
                        <MdHistory className={styles.pillarIcon} />
                        <h3>Real-time Monitoring</h3>
                        <p>Our system tracks all on-chain activity to detect and alert you to suspicious events instantly.</p>
                    </div>
                </div>
            </section>

            {/* Compliance Section */}
            <section className={styles.complianceSection}>
                <div className={styles.complianceContent}>
                    <h2>Transparency & Standards</h2>
                    <p>
                        While in Beta, we are committed to building toward the world's most 
                        rigorous security standards. Our infrastructure is designed to 
                        facilitate future audits and institutional-grade compliance.
                    </p>
                    <div className={styles.certGrid}>
                        <div className={styles.certItem}><MdVerified /> Multi-Chain Architecture</div>
                        <div className={styles.certItem}><MdVerified /> Hardware Security Options</div>
                        <div className={styles.certItem}><MdVerified /> GDPR-First Privacy</div>
                        <div className={styles.certItem}><MdVerified /> Scalable to ISO/SOC Standards</div>
                    </div>
                </div>
            </section>

            {/* Trust CTA */}
            <section className={styles.ctaSection}>
                <div className={styles.ctaCard}>
                    <h2>Be part of our Beta launch.</h2>
                    <p>Join the next generation of merchants building secure crypto payment workflows.</p>
                    <div className={styles.ctaActions}>
                        <button className={styles.ctaPrimary}>Get Beta Access</button>
                    </div>
                </div>
            </section>
        </div>
    )
}

export default PublicSecurityPage
