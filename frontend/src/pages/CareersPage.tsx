import React from 'react'
import styles from './CareersPage.module.css'

const CareersPage: React.FC = () => {
  const openPositions = [
    {
      title: 'Senior Backend Engineer',
      department: 'Engineering',
      location: 'Remote',
      type: 'Full-time',
      description: 'Build scalable payment infrastructure and blockchain integrations.'
    },
    {
      title: 'Frontend Developer',
      department: 'Engineering',
      location: 'Remote',
      type: 'Full-time',
      description: 'Create beautiful user interfaces and seamless user experiences.'
    },
    {
      title: 'DevOps Engineer',
      department: 'Engineering',
      location: 'Remote',
      type: 'Full-time',
      description: 'Manage infrastructure, deployments, and system reliability.'
    },
    {
      title: 'Product Manager',
      department: 'Product',
      location: 'Remote',
      type: 'Full-time',
      description: 'Drive product strategy and roadmap for crypto payment solutions.'
    }
  ]

  return (
    <div className={styles.careersPage}>
      <div className={styles.container}>
        {/* Hero Section */}
        <section className={styles.hero}>
          <h1>Join Our Mission</h1>
          <p>Help us build the future of cryptocurrency payments and make digital transactions accessible to everyone.</p>
        </section>

        {/* Values Section */}
        <section className={styles.values}>
          <h2>Why Work With Us</h2>
          <div className={styles.valuesGrid}>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}>
                <i className="fas fa-rocket"></i>
              </div>
              <h3>Innovation First</h3>
              <p>Work on cutting-edge blockchain technology and shape the future of digital payments.</p>
            </div>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}>
                <i className="fas fa-globe"></i>
              </div>
              <h3>Remote Culture</h3>
              <p>Work from anywhere with flexible hours and a strong remote-first culture.</p>
            </div>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}>
                <i className="fas fa-users"></i>
              </div>
              <h3>Great Team</h3>
              <p>Collaborate with talented individuals who are passionate about technology and growth.</p>
            </div>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}>
                <i className="fas fa-chart-line"></i>
              </div>
              <h3>Growth Opportunities</h3>
              <p>Advance your career with learning opportunities, mentorship, and leadership roles.</p>
            </div>
          </div>
        </section>

        {/* Open Positions */}
        <section className={styles.positions}>
          <h2>Open Positions</h2>
          <div className={styles.positionsList}>
            {openPositions.map((position, index) => (
              <div key={index} className={styles.positionCard}>
                <div className={styles.positionHeader}>
                  <h3>{position.title}</h3>
                  <div className={styles.positionMeta}>
                    <span className={styles.department}>{position.department}</span>
                    <span className={styles.location}>{position.location}</span>
                    <span className={styles.type}>{position.type}</span>
                  </div>
                </div>
                <p>{position.description}</p>
                <button className={styles.applyBtn}>Apply Now</button>
              </div>
            ))}
          </div>
        </section>

        {/* Benefits Section */}
        <section className={styles.benefits}>
          <h2>Benefits & Perks</h2>
          <div className={styles.benefitsList}>
            <div className={styles.benefit}>
              <i className="fas fa-heart"></i>
              <span>Comprehensive health insurance</span>
            </div>
            <div className={styles.benefit}>
              <i className="fas fa-plane"></i>
              <span>Unlimited PTO policy</span>
            </div>
            <div className={styles.benefit}>
              <i className="fas fa-laptop"></i>
              <span>Top-tier equipment and tools</span>
            </div>
            <div className={styles.benefit}>
              <i className="fas fa-graduation-cap"></i>
              <span>Learning and development budget</span>
            </div>
            <div className={styles.benefit}>
              <i className="fas fa-coins"></i>
              <span>Competitive salary and equity</span>
            </div>
            <div className={styles.benefit}>
              <i className="fas fa-home"></i>
              <span>Remote work stipend</span>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className={styles.cta}>
          <h2>Don't See Your Role?</h2>
          <p>We're always looking for talented individuals. Send us your resume and let us know how you'd like to contribute.</p>
          <a href="mailto:careers@fiddupay.com" className={styles.ctaBtn}>Get In Touch</a>
        </section>
      </div>
    </div>
  )
}

export default CareersPage
