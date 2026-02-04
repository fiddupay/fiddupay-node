import React, { useState, useEffect, useRef } from 'react';
import { useParams, useLocation } from 'react-router-dom';
import styles from '@/styles/pages/DocsPage.module.css';
import mobileStyles from '@/styles/pages/DocsPageMobile.module.css';
import CodeSnippet from '../components/docs/CodeSnippet';
import DocsSidebar from '../components/docs/DocsSidebar';
import ApiSection from '../components/docs/ApiSection';
import { API_DATA, Endpoint } from './docs/ApiData';

const DocsPage: React.FC = () => {
  const { sectionId } = useParams<{ sectionId: string }>();
  const location = useLocation();
  const [activeSection, setActiveSection] = useState(sectionId || 'getting-started');
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const sectionRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
  const isScrollingRef = useRef(false);

  // Scroll to section/hash logic... (unchanged)
  useEffect(() => {
    if (sectionId) {
      setActiveSection(sectionId);
      // Allow DOM to render then scroll
      setTimeout(() => {
        const hash = location.hash.replace('#', '');
        const targetId = hash || sectionId;
        const element = document.getElementById(targetId);
        if (element) {
          isScrollingRef.current = true;
          element.scrollIntoView({ behavior: 'smooth' });
          setTimeout(() => { isScrollingRef.current = false; }, 1000);
        }
      }, 100);
    }
  }, [sectionId, location.hash]);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        // Find the "most visible" section
        let maxRatio = 0;
        let bestTarget = '';

        entries.forEach((entry) => {
          if (entry.intersectionRatio > maxRatio) {
            maxRatio = entry.intersectionRatio;
            bestTarget = entry.target.id;
          }
        });

        if (bestTarget && !isScrollingRef.current) {
          if (activeSection !== bestTarget) {
            setActiveSection(bestTarget);

            // Determine if target is a top-level section or endpoint
            const section = API_DATA.find(s => s.id === bestTarget);
            const endpoint = API_DATA.flatMap(s => s.endpoints).find(e => e.id === bestTarget);

            let newUrl = `/docs/${bestTarget}`; // Default fallback

            if (section) {
              // It's a top-level section
              newUrl = `/docs/${section.id}`;
            } else if (endpoint) {
              // It's an endpoint, find its parent section
              const parentSection = API_DATA.find(s => s.endpoints.some(e => e.id === bestTarget));
              if (parentSection) {
                newUrl = `/docs/${parentSection.id}#${bestTarget}`;
              }
            }

            // Update URL silently
            window.history.replaceState(null, '', newUrl);
          }
        }
      },
      { threshold: [0.1, 0.5, 0.8], rootMargin: '-80px 0px -20% 0px' }
    );

    Object.values(sectionRefs.current).forEach((ref) => {
      if (ref) observer.observe(ref);
    });

    return () => observer.disconnect();
  }, []);

  const scrollToSection = (id: string) => {
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <div className={`${styles.docsPage} ${mobileStyles.docsPage}`}>

      {/* Mobile Header (Visible only on mobile via CSS) */}
      <div className={mobileStyles.mobileHeader}>
        <div className={mobileStyles.logoSection}>
          <div className={mobileStyles.burgerIcon}>
            <div className={mobileStyles.bar}></div>
            <div className={mobileStyles.bar}></div>
            <div className={mobileStyles.bar}></div>
          </div>
        </div>
        <div className={mobileStyles.headerActions}>
          <button className={mobileStyles.searchBtn} onClick={() => alert('Search functionality coming soon!')}>
            <i className="fas fa-search"></i> Search
          </button>
          <button
            className={mobileStyles.menuBtn}
            onClick={() => setIsMobileMenuOpen(true)}
          >
            <i className="fas fa-bars"></i> Menu
          </button>
        </div>
      </div>

      {/* Mobile Menu Overlay Backdrop */}
      {isMobileMenuOpen && (
        <div
          className={mobileStyles.overlayBackdrop}
          onClick={() => setIsMobileMenuOpen(false)}
        />
      )}

      {/* Column 1: Navigation */}
      <aside className={`${styles.sidebar} ${mobileStyles.sidebar} ${isMobileMenuOpen ? mobileStyles.open : ''}`}>
        <div className={mobileStyles.mobileMenuHeader}>
          <button
            className={mobileStyles.closeBtn}
            onClick={() => setIsMobileMenuOpen(false)}
          >
            <i className="fas fa-times"></i> Close
          </button>
        </div>
        <DocsSidebar
          apiData={API_DATA}
          activeSection={activeSection}
          scrollToSection={(id) => {
            scrollToSection(id);
            setIsMobileMenuOpen(false);
          }}
        />
      </aside>

      {/* Column 2 & 3 Wrap */}
      <div className={`${styles.contentArea} ${mobileStyles.contentArea}`}>
        <main className={`${styles.mainContent} ${mobileStyles.mainContent}`}>
          {API_DATA.map((section) => (
            <ApiSection
              key={section.id}
              section={section}
              sectionRefs={sectionRefs}
            />
          ))}

          <div className={styles.infoBox}>
            <p>
              Looking for older documentation? Check our <a href="https://github.com/fiddupay/fiddupay-node/releases" target="_blank" rel="noopener noreferrer">release notes</a> for version changes and migration guides.
            </p>
          </div>
        </main>

        {/* Column 3: Code Samples */}
        {(() => {
          const item = API_DATA.flatMap(s => [s, ...s.endpoints]).find(e => e.id === activeSection);
          const hasCode = item && (item as Endpoint).request;

          return (
            <aside
              className={`${styles.codeColumn} ${mobileStyles.codeColumn}`}
              style={{ display: hasCode ? 'block' : 'none' }}
            >
              <div className={`${styles.codeSticky} ${mobileStyles.codeSticky}`}>
                {hasCode && (
                  <div key={activeSection}>
                    <CodeSnippet
                      request={(item as Endpoint).request}
                      response={(item as Endpoint).response}
                      method={(item as Endpoint).method}
                      path={(item as Endpoint).path}
                    />
                  </div>
                )}
              </div>
            </aside>
          );
        })()}
      </div>
    </div>
  );
};

export default DocsPage;
