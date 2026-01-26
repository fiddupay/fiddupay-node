import React from 'react'
import styles from './BlogPage.module.css'

const BlogPage: React.FC = () => {
  return (
    <div className={styles.blogPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1>FidduPay Blog</h1>
          <p>Insights, updates, and guides from the crypto payment experts</p>
        </div>

        <div className={styles.posts}>
          <article className={styles.post}>
            <div className={styles.postImage}>
              <i className="fas fa-shield-alt"></i>
            </div>
            <div className={styles.postContent}>
              <h2>The Future of Crypto Payment Security</h2>
              <p className={styles.postMeta}>January 25, 2026 • 5 min read</p>
              <p>Exploring the latest security innovations in cryptocurrency payment processing and how FidduPay stays ahead of threats.</p>
              <a href="#" className={styles.readMore}>Read More</a>
            </div>
          </article>

          <article className={styles.post}>
            <div className={styles.postImage}>
              <i className="fas fa-chart-line"></i>
            </div>
            <div className={styles.postContent}>
              <h2>Multi-Chain Payments: The New Standard</h2>
              <p className={styles.postMeta}>January 20, 2026 • 7 min read</p>
              <p>Why businesses are adopting multi-blockchain payment solutions and how to choose the right networks for your needs.</p>
              <a href="#" className={styles.readMore}>Read More</a>
            </div>
          </article>

          <article className={styles.post}>
            <div className={styles.postImage}>
              <i className="fas fa-code"></i>
            </div>
            <div className={styles.postContent}>
              <h2>Integrating FidduPay: A Developer's Guide</h2>
              <p className={styles.postMeta}>January 15, 2026 • 10 min read</p>
              <p>Step-by-step tutorial on integrating cryptocurrency payments into your application using our Node.js SDK.</p>
              <a href="#" className={styles.readMore}>Read More</a>
            </div>
          </article>
        </div>
      </div>
    </div>
  )
}

export default BlogPage
