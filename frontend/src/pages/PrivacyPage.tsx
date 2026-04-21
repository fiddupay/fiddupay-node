import React from 'react'
import styles from '@/styles/pages/PrivacyPage.module.css'

const PrivacyPage: React.FC = () => {
    return (
        <div className={styles.legalPage}>
            {/* Ambient Glow */}
            <div className={styles.ambientGlowContainer}>
                <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
                <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
            </div>

            <div className={styles.container}>
                <header className={`${styles.header} animate-fade-in-up`}>
                    <div className={styles.badge}>Security & Trust</div>
                    <h1 className={styles.title}>Privacy Policy</h1>
                    <p className={styles.subtitle}>Last updated: January 24, 2026</p>
                </header>

                <div className={styles.contentGrid}>
                    <aside className={styles.sidebar}>
                        <nav className={styles.stickyNav}>
                            <h3>Sections</h3>
                            <ul>
                                <li><a href="#introduction">Introduction</a></li>
                                <li><a href="#data-collection">Data Collection</a></li>
                                <li><a href="#usage">Usage of Data</a></li>
                                <li><a href="#protection">Protection Measures</a></li>
                                <li><a href="#third-parties">Third Parties</a></li>
                                <li><a href="#rights">Your Rights</a></li>
                            </ul>
                        </nav>
                    </aside>

                    <main className={styles.mainContent}>
                        <section id="introduction" className={styles.section}>
                            <h2>1. Introduction</h2>
                            <p>
                                At FidduPay, we respect your privacy and are committed to protecting your personal data. 
                                This privacy policy is designed to align with the <b>Nigeria Data Protection Act (NDPA) 2023</b>. 
                                As we operate in Public Beta, we are actively preparing for formal registration with the 
                                <b>Nigeria Data Protection Commission (NDPC)</b> as a data controller and processor.
                            </p>
                        </section>

                        <section id="data-collection" className={styles.section}>
                            <h2>2. Data Collection</h2>
                            <p>
                                We collect various types of information to provide and improve our service to you:
                            </p>
                            <ul>
                                <li><strong>Identity Data:</strong> First name, last name, username or similar identifier.</li>
                                <li><strong>Contact Data:</strong> Billing address, email address and telephone numbers.</li>
                                <li><strong>Financial Data:</strong> Wallet addresses, bank account and payment card details.</li>
                                <li><strong>Technical Data:</strong> IP address, login data, browser type and version, time zone setting and location.</li>
                            </ul>
                        </section>

                        <section id="usage" className={styles.section}>
                            <h2>3. Usage of Data</h2>
                            <p>
                                We use your personal data only when the law allows us to. Most commonly, we will use your personal data in the following circumstances:
                            </p>
                            <ul>
                                <li>Where we need to perform the contract we are about to enter into or have entered into with you.</li>
                                <li>Where it is necessary for our legitimate interests and your interests and fundamental rights do not override those interests.</li>
                                <li>Where we need to comply with a legal or regulatory obligation.</li>
                            </ul>
                        </section>

                        <section id="protection" className={styles.section}>
                            <h2>4. Protection Measures</h2>
                            <p>
                                We have put in place appropriate security measures to prevent your personal data from being accidentally lost, used or accessed in an unauthorized way, altered or disclosed. 
                            </p>
                            <p>
                                All your data is encrypted using AES-256 and transmitted via TLS 1.3. We limit access to your personal data to those employees, agents, contractors and other third parties who have a business need to know.
                            </p>
                        </section>

                        <section id="third-parties" className={styles.section}>
                            <h2>5. Third Parties</h2>
                            <p>
                                We may share your personal data with internal and external third parties such as:
                            </p>
                            <ul>
                                <li>Service providers who provide infrastructure and IT support.</li>
                                <li>Professional advisers including lawyers, bankers, auditors and insurers.</li>
                                <li>Regulators and other authorities who require reporting of processing activities.</li>
                            </ul>
                        </section>

                        <section id="rights" className={styles.section}>
                            <h2>6. Your Rights</h2>
                            <p>
                                Under certain circumstances, you have rights under data protection laws in relation to your personal data, including the right to request access, correction, erasure, restriction, and more.
                            </p>
                            <div className={styles.contactLegal}>
                                <p>For any privacy-related inquiries, please contact: <strong>privacy@fiddupay.com</strong></p>
                            </div>
                        </section>
                    </main>
                </div>
            </div>
        </div>
    )
}

export default PrivacyPage
