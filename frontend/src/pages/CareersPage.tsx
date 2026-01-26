import React from 'react'
import styles from './CareersPage.module.css'

const CareersPage: React.FC = () => {
  return (
    <div className={styles.careersPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1>Join the FidduPay Team</h1>
          <p>Help us build the future of cryptocurrency payments</p>
        </div>

        <div className={styles.content}>
          <section className={styles.section}>
            <h2>Why Work at FidduPay?</h2>
            <div className={styles.benefits}>
              <div className={styles.benefit}>
                <i className="fas fa-rocket"></i>
                <h3>Cutting-Edge Technology</h3>
                <p>Work with the latest blockchain and fintech technologies</p>
              </div>
              <div className={styles.benefit}>
                <i className="fas fa-users"></i>
                <h3>Amazing Team</h3>
                <p>Collaborate with talented engineers and crypto experts</p>
              </div>
              <div className={styles.benefit}>
                <i className="fas fa-chart-line"></i>
                <h3>Growth Opportunities</h3>
                <p>Advance your career in the rapidly growing crypto industry</p>
              </div>
            </div>
          </section>

          <section className={styles.section}>
            <h2>Open Positions</h2>
            <div className={styles.jobs}>
              <div className={styles.job}>
                <h3>Senior Blockchain Developer</h3>
                <p>Remote • Full-time</p>
                <p>Build and maintain our multi-chain payment infrastructure</p>
              </div>
              <div className={styles.job}>
                <h3>Frontend Engineer</h3>
                <p>Remote • Full-time</p>
                <p>Create beautiful, user-friendly payment interfaces</p>
              </div>
              <div className={styles.job}>
                <h3>DevOps Engineer</h3>
                <p>Remote • Full-time</p>
                <p>Scale our infrastructure to handle millions of transactions</p>
              </div>
            </div>
          </section>

          <section className={styles.section}>
            <h2>Ready to Apply?</h2>
            <p>Send your resume and portfolio to <a href="mailto:careers@fiddupay.com">careers@fiddupay.com</a></p>
          </section>
        </div>
      </div>
    </div>
  )
}

export default CareersPage
