import React from 'react'
import { MdShield, MdHistory, MdVerified, MdFingerprint } from 'react-icons/md'
import styles from '@/styles/pages/PublicSecurityPage.module.css'

const PublicSecurityPage: React.FC = () => {
    return (
        <div className={styles.publicSecurityPage}>
            {/* Ambient Glow */}
            <div className={styles.ambientGlowContainer}>
                <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
                <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
            </div>

            <div className={styles.container}>
                {/* Hero Section */}
                <section className={`${styles.heroSection} animate-fade-in-up`}>
                    <div className={styles.heroContent}>
                        <div className={styles.badge}>
                            <MdShield />
                            <span>Enterprise Core</span>
                        </div>
                        <h1 className={styles.title}>Secure, Transparent, <span className={styles.gradientText}>Future-Ready</span></h1>
                        <p className={styles.subtitle}>
                            FidduPay architecture is built on a security-first foundation. We combine multi-layered encryption with a robust multi-chain infrastructure to safeguard every transaction from day one.
                        </p>
                        <div className={styles.heroActions}>
                            <button className={styles.primaryBtn}>Review Security Specs</button>
                            <button className={styles.secondaryBtn}>Infrastructure Status</button>
                        </div>
                    </div>
                    <div className={styles.heroVisual}>
                        <div className={styles.glassCard}>
                            <div className={styles.radarContainer}>
                                <div className={styles.radarLine}></div>
                                <MdShield className={styles.radarIcon} />
                            </div>
                            <div className={styles.glassText}>
                                <strong>L3 Threat Detection</strong>
                                <span>Scanning blockchain nodes...</span>
                            </div>
                        </div>
                        <div className={styles.heroGlow}></div>
                    </div>
                </section>

                {/* Core Pillars */}
                <section className={styles.pillarsSection}>
                    <div className={styles.sectionHeader}>
                        <h2>Institutional Trust Pillars</h2>
                        <p>Our security stack is designed for scale, transparency, and uncompromising protection.</p>
                    </div>
                    <div className={styles.pillarsGrid}>
                        <div className={`${styles.pillarCard} animate-fade-in-up`} style={{ animationDelay: '0.1s' }}>
                            <div className={styles.pillarIconBox}><MdShield /></div>
                            <h3>AES-256 Encryption</h3>
                            <p>All merchant data and sensitive transaction payloads are protected by military-grade encryption at rest and TLS 1.3 in transit.</p>
                        </div>
                        <div className={`${styles.pillarCard} animate-fade-in-up`} style={{ animationDelay: '0.2s' }}>
                            <div className={styles.pillarIconBox}><MdFingerprint /></div>
                            <h3>Identity Protection</h3>
                            <p>Granular access controls, mandatory 2FA, and session-based biometric options prevent unauthorized entry into your dashboard.</p>
                        </div>
                        <div className={`${styles.pillarCard} animate-fade-in-up`} style={{ animationDelay: '0.3s' }}>
                            <div className={styles.pillarIconBox}><MdHistory /></div>
                            <h3>Real-time Monitoring</h3>
                            <p>Our proprietary L3 monitors track all on-chain activity to detect and bridge deviations before they impact your balance.</p>
                        </div>
                    </div>
                </section>

                {/* Compliance Section */}
                <section className={styles.complianceSection}>
                    <div className={styles.complianceCard}>
                        <div className={styles.complianceText}>
                            <h2>Transparency & Standards</h2>
                            <p>
                                While in Beta, we are committed to building toward the world's most 
                                rigorous security standards. Our infrastructure is designed to 
                                facilitate future SOC 2 Type II audits and institutional-grade compliance.
                            </p>
                            <div className={styles.certGrid}>
                                <div className={styles.certItem}><MdVerified /> Multi-Chain Architecture</div>
                                <div className={styles.certItem}><MdVerified /> Hardware Security Options</div>
                                <div className={styles.certItem}><MdVerified /> GDPR-First Privacy</div>
                                <div className={styles.certItem}><MdVerified /> Scalable to ISO/SOC Standards</div>
                            </div>
                        </div>
                        <div className={styles.complianceVisual}>
                            <i className="fas fa-file-shield"></i>
                        </div>
                    </div>
                </section>

                {/* Trust CTA */}
                <section className={styles.ctaSection}>
                    <div className={styles.ctaCard}>
                        <div className={styles.ctaGlow}></div>
                        <h2>Join the Secure Beta</h2>
                        <p>Be part of the next generation of merchants building secure crypto payment workflows with FidduPay.</p>
                        <div className={styles.ctaActions}>
                            <button className={styles.ctaPrimary}>Get Early Access</button>
                            <button className={styles.ctaSecondary}>Contact Security Team</button>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    )
}

export default PublicSecurityPage
