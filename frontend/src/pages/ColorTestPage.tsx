import React from 'react'
import styles from './ColorTestPage.module.css'

const ColorTestPage: React.FC = () => {
  return (
    <div className={styles.testPage}>
      <div className={styles.container}>
        <h1 className={styles.title}>FidduPay Color System Test</h1>
        
        <section className={styles.section}>
          <h2>Background Colors</h2>
          <div className={styles.colorGrid}>
            <div className={styles.colorBox} style={{background: '#ffffff', border: '1px solid #e5e7eb'}}>
              <span style={{color: '#000000'}}>White Background (#ffffff)</span>
              <span style={{color: '#000000'}}>Black Text (#000000)</span>
            </div>
            <div className={styles.colorBox} style={{background: '#f3f4f6'}}>
              <span style={{color: '#000000'}}>Light Gray Background (#f3f4f6)</span>
              <span style={{color: '#000000'}}>Black Text (#000000)</span>
            </div>
          </div>
        </section>

        <section className={styles.section}>
          <h2>Text Colors</h2>
          <div className={styles.textTests}>
            <p style={{color: '#000000'}}>Primary Text - Black (#000000)</p>
            <p style={{color: '#6b7280'}}>Secondary Text - Gray (#6b7280)</p>
            <p style={{color: '#1e40af'}}>Brand Blue Text (#1e40af)</p>
          </div>
        </section>

        <section className={styles.section}>
          <h2>Buttons</h2>
          <div className={styles.buttonTests}>
            <button style={{background: '#1e40af', color: '#ffffff', border: 'none', padding: '12px 24px', borderRadius: '6px'}}>
              Primary Button (Blue bg, White text)
            </button>
            <button style={{background: '#ffffff', color: '#000000', border: '1px solid #e5e7eb', padding: '12px 24px', borderRadius: '6px'}}>
              Secondary Button (White bg, Black text)
            </button>
            <button style={{background: '#059669', color: '#ffffff', border: 'none', padding: '12px 24px', borderRadius: '6px'}}>
              Success Button (Green bg, White text)
            </button>
          </div>
        </section>

        <section className={styles.section}>
          <h2>Status Badges</h2>
          <div className={styles.badgeTests}>
            <span style={{background: '#059669', color: '#ffffff', padding: '4px 12px', borderRadius: '12px', fontSize: '12px'}}>
              Success Badge
            </span>
            <span style={{background: '#f59e0b', color: '#ffffff', padding: '4px 12px', borderRadius: '12px', fontSize: '12px'}}>
              Warning Badge
            </span>
            <span style={{background: '#1e40af', color: '#ffffff', padding: '4px 12px', borderRadius: '12px', fontSize: '12px'}}>
              Info Badge
            </span>
          </div>
        </section>

        <section className={styles.section}>
          <h2>Cards</h2>
          <div className={styles.cardTest} style={{background: '#ffffff', border: '1px solid #e5e7eb', borderRadius: '8px', padding: '20px'}}>
            <h3 style={{color: '#000000', marginBottom: '8px'}}>Card Title (Black)</h3>
            <p style={{color: '#6b7280'}}>Card description text in gray for secondary information.</p>
            <button style={{background: '#1e40af', color: '#ffffff', border: 'none', padding: '8px 16px', borderRadius: '4px', marginTop: '12px'}}>
              Card Action
            </button>
          </div>
        </section>

        <section className={styles.section}>
          <h2>Form Elements</h2>
          <div className={styles.formTests}>
            <label style={{color: '#000000', fontWeight: '600', display: 'block', marginBottom: '4px'}}>
              Input Label (Black)
            </label>
            <input 
              type="text" 
              placeholder="Input field"
              style={{
                background: '#ffffff', 
                color: '#000000', 
                border: '1px solid #e5e7eb', 
                padding: '8px 12px', 
                borderRadius: '4px',
                width: '200px'
              }}
            />
          </div>
        </section>

        <section className={styles.section}>
          <h2>Links</h2>
          <div className={styles.linkTests}>
            <a href="#" style={{color: '#1e40af', textDecoration: 'none'}}>
              Brand Blue Link (#1e40af)
            </a>
          </div>
        </section>

        <div className={styles.summary}>
          <h2>Color Rules Summary</h2>
          <ul>
            <li><strong>Background:</strong> Always white (#ffffff) or light gray (#f3f4f6)</li>
            <li><strong>Text:</strong> Always black (#000000) or gray (#6b7280)</li>
            <li><strong>Brand Blue:</strong> #1e40af for buttons, links, branding</li>
            <li><strong>Brand Green:</strong> #059669 for success states</li>
            <li><strong>Brand Gold:</strong> #f59e0b for warnings</li>
            <li><strong>Never:</strong> White text on white background</li>
            <li><strong>Never:</strong> Same color on same color</li>
          </ul>
        </div>
      </div>
    </div>
  )
}

export default ColorTestPage
