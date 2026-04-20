import React from 'react'
import styles from '@/styles/pages/BlogPage.module.css'

const BlogPage: React.FC = () => {
  const posts = [
    {
      id: 1,
      title: 'The Future of Crypto Payment Security',
      category: 'Security',
      date: 'January 25, 2026',
      readTime: '5 min read',
      excerpt: 'Exploring the latest security innovations in cryptocurrency payment processing and how FidduPay stays ahead of threats with L3 monitoring.',
      icon: 'fa-shield-halved',
      color: 'var(--primary)'
    },
    {
      id: 2,
      title: 'Multi-Chain Payments: The New Industry Standard',
      category: 'Ecosystem',
      date: 'January 20, 2026',
      readTime: '7 min read',
      excerpt: 'Why businesses are adopting multi-blockchain payment solutions and how to choose the right networks for your specific customer base.',
      icon: 'fa-chart-line',
      color: 'var(--secondary)'
    },
    {
      id: 3,
      title: 'Integrating FidduPay: A Developer\'s Deep Dive',
      category: 'Engineering',
      date: 'January 15, 2026',
      readTime: '10 min read',
      excerpt: 'A comprehensive step-by-step tutorial on integrating cryptocurrency payments into your application using our high-performance Node.js SDK.',
      icon: 'fa-code',
      color: '#818cf8'
    }
  ]

  return (
    <div className={styles.blogPage}>
      {/* Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <div className={`${styles.header} animate-fade-in-up`}>
          <div className={styles.badge}>FidduPay Insights</div>
          <h1>The <span className={styles.gradientText}>Financial Engine</span> Blog</h1>
          <p>Deep dives into blockchain infrastructure, payment security, and the future of decentralized finance.</p>
        </div>

        <div className={styles.blogGrid}>
          {posts.map((post, index) => (
            <article key={post.id} className={`${styles.postCard} animate-fade-in-up`} style={{ animationDelay: `${index * 0.15}s` }}>
              <div className={styles.cardVisual} style={{ background: `linear-gradient(135deg, ${post.color}22, transparent)` }}>
                <div className={styles.iconBox} style={{ color: post.color, boxShadow: `0 0 20px ${post.color}33` }}>
                  <i className={`fas ${post.icon}`}></i>
                </div>
                <div className={styles.categoryBadge}>{post.category}</div>
              </div>
              
              <div className={styles.cardContent}>
                <div className={styles.metaRow}>
                  <span>{post.date}</span>
                  <span className={styles.dot}></span>
                  <span>{post.readTime}</span>
                </div>
                <h2 className={styles.postTitle}>{post.title}</h2>
                <p className={styles.postExcerpt}>{post.excerpt}</p>
                
                <div className={styles.cardFooter}>
                  <a href="#" className={styles.readMore}>
                    Read Article
                    <i className="fas fa-arrow-right"></i>
                  </a>
                </div>
              </div>
            </article>
          ))}
        </div>

        {/* Newsletter / CTA */}
        <section className={`${styles.newsletter} animate-fade-in`}>
          <div className={styles.newsletterContent}>
            <h2>Stay ahead of the curve</h2>
            <p>Get the latest engineering updates and crypto market insights delivered to your inbox.</p>
            <form className={styles.subscribeForm} onSubmit={(e) => e.preventDefault()}>
              <input type="email" placeholder="Enter your email" className={styles.emailInput} />
              <button type="submit" className={styles.subscribeBtn}>Join 5,000+ Readers</button>
            </form>
          </div>
        </section>
      </div>
    </div>
  )
}

export default BlogPage
