import React from 'react'
import styles from '@/styles/pages/CareersPage.module.css'

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
       {/* Ambient Glow */}
       <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        {/* Hero Section */}
        <section className={`${styles.hero} animate-fade-in-up`}>
          <div className={styles.badge}>Join the Revolution</div>
          <h1>Help Us Build the <span className={styles.gradientText}>Future of Money</span></h1>
          <p>We're looking for passionate individuals to help us make digital transactions accessible, secure, and instant for everyone on Earth.</p>
        </section>

        {/* Values Section */}
        <section className={styles.values}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>Why FidduPay?</h2>
            <p className={styles.sectionSubtitle}>We're more than just a payment gateway. We're a team of visionaries redefining financial sovereignty.</p>
          </div>
          <div className={styles.valuesGrid}>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}><i className="fas fa-rocket"></i></div>
              <h3>Innovation First</h3>
              <p>Work on the absolute edge of blockchain technology and L3 monitoring systems.</p>
            </div>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}><i className="fas fa-globe"></i></div>
              <h3>Remote Culture</h3>
              <p>Work from anywhere. We believe talent is global and productivity is measured by impact.</p>
            </div>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}><i className="fas fa-users"></i></div>
              <h3>High Autonomy</h3>
              <p>We hire experts and trust them. Own your projects from conception to production.</p>
            </div>
            <div className={styles.valueCard}>
              <div className={styles.valueIcon}><i className="fas fa-chart-line"></i></div>
              <h3>Fast Growth</h3>
              <p>Join a scaling startup where your contributions directly shape the company roadmap.</p>
            </div>
          </div>
        </section>

        {/* Open Positions */}
        <section className={styles.positions}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>Open Positions</h2>
            <p className={styles.sectionSubtitle}>Find your next challenge in our growing engineering and product teams.</p>
          </div>
          <div className={styles.positionsList}>
            {openPositions.map((position, index) => (
              <div key={index} className={`${styles.positionCard} animate-fade-in-up`} style={{ animationDelay: `${index * 0.1}s` }}>
                <div className={styles.positionInfo}>
                  <div className={styles.positionTitleRow}>
                    <h3>{position.title}</h3>
                    <div className={styles.positionBadges}>
                      <span className={styles.deptBadge}>{position.department}</span>
                      <span className={styles.typeBadge}>{position.type}</span>
                    </div>
                  </div>
                  <p className={styles.positionDesc}>{position.description}</p>
                  <div className={styles.positionMeta}>
                    <span><i className="fas fa-map-marker-alt"></i> {position.location}</span>
                    <span><i className="fas fa-clock"></i> Posted 2 days ago</span>
                  </div>
                </div>
                <button className={styles.applyBtn}>
                  Apply Now
                  <i className="fas fa-external-link-alt"></i>
                </button>
              </div>
            ))}
          </div>
        </section>

        {/* Benefits Section */}
        <section className={styles.benefits}>
          <div className={styles.benefitsCard}>
            <h2>Perks & Benefits</h2>
            <div className={styles.benefitsGrid}>
              <div className={styles.benefitItem}>
                <i className="fas fa-heart"></i>
                <span>Premium Health Coverage</span>
              </div>
              <div className={styles.benefitItem}>
                <i className="fas fa-plane"></i>
                <span>Flexible PTO Policy</span>
              </div>
              <div className={styles.benefitItem}>
                <i className="fas fa-laptop"></i>
                <span>Home Office Stipend</span>
              </div>
              <div className={styles.benefitItem}>
                <i className="fas fa-graduation-cap"></i>
                <span>L&D Annual Budget</span>
              </div>
              <div className={styles.benefitItem}>
                <i className="fas fa-coins"></i>
                <span>Equity & Token Options</span>
              </div>
              <div className={styles.benefitItem}>
                <i className="fas fa-birthday-cake"></i>
                <span>Regular Team Offsites</span>
              </div>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className={styles.cta}>
          <div className={styles.ctaContent}>
            <h2>Don't see your role?</h2>
            <p>We're always looking for geniuses who don't fit into a box. Send us an open application.</p>
            <a href="mailto:careers@fiddupay.com" className={styles.ctaBtn}>
              Send Open Application
              <i className="fas fa-paper-plane"></i>
            </a>
          </div>
        </section>
      </div>
    </div>
  )
}

export default CareersPage
