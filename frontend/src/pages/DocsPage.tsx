import React, { useState, useEffect, useRef } from 'react';
import { useParams, useLocation, Link } from 'react-router-dom';
import styles from '@/styles/pages/DocsPage.module.css';
import mobileStyles from '@/styles/pages/DocsPageMobile.module.css';
import CodeSnippet from '../components/docs/CodeSnippet';
import DocsSidebar from '../components/docs/DocsSidebar';
import ApiSection from '../components/docs/ApiSection';
import { Endpoint, API_DATA } from './docs/ApiData';

const DocsPage: React.FC = () => {
  const { sectionId } = useParams<{ sectionId: string }>();
  const location = useLocation();
  const [activeSection, setActiveSection] = useState(sectionId || 'getting-started');
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const sectionRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
  const isScrollingRef = useRef(false);
  const currentRatios = useRef<{ [key: string]: number }>({});

  // Scroll to section/hash logic...
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
        // Update ratios for changed entries
        entries.forEach((entry) => {
          currentRatios.current[entry.target.id] = entry.intersectionRatio;
        });

        // Determine best target from ALL current ratios
        let bestTarget = '';
        let maxScore = -1;

        Object.entries(currentRatios.current).forEach(([id, ratio]) => {
          if (ratio <= 0) return;

          const isEndpoint = API_DATA.some(s => s.endpoints.some(e => e.id === id));
          // Boost endpoints to ensure they override parent sections when visible
          const score = ratio + (isEndpoint ? 2.0 : 0);

          if (score > maxScore) {
            maxScore = score;
            bestTarget = id;
          }
        });

        // URL Verification Logic
        // We calculate what the URL *should* be based on bestTarget
        if (bestTarget && !isScrollingRef.current) {

          // Calculate canonical URL
          const section = API_DATA.find(s => s.id === bestTarget);
          const endpoint = API_DATA.flatMap(s => s.endpoints).find(e => e.id === bestTarget);

          let newUrl = `/docs/${bestTarget}`;

          if (section) {
            newUrl = `/docs/${section.id}`;
          } else if (endpoint) {
            const parentSection = API_DATA.find(s => s.endpoints.some(e => e.id === bestTarget));
            if (parentSection) {
              newUrl = `/docs/${parentSection.id}#${bestTarget}`;
            }
          }

          // Check if we need to update URL (if section changed OR if URL is messy)
          const currentPath = window.location.pathname;
          const currentHash = window.location.hash;
          const urlMismatch = (currentPath + currentHash) !== newUrl;

          if (activeSection !== bestTarget) {
            setActiveSection(bestTarget);
          }

          if ((activeSection !== bestTarget || urlMismatch) && newUrl) {
            window.history.replaceState(null, '', newUrl);
          }
        }
      },
      { threshold: [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0], rootMargin: '-10px 0px -50% 0px' }
    );

    Object.values(sectionRefs.current).forEach((ref) => {
      if (ref) observer.observe(ref);
    });

    return () => observer.disconnect();
  }, [activeSection]);

  const scrollToSection = (id: string) => {
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <div className={`${styles.docsPage} ${mobileStyles.docsPage}`}>

      {/* Desktop Header (Hidden on Mobile) */}
      <header className={styles.desktopHeader}>
        <Link to="/" className={styles.headerLogo}>FidduPay</Link>

        <div className={styles.searchContainer}>
          <i className={`fas fa-search ${styles.searchIcon}`}></i>
          <input
            type="text"
            placeholder="Search documentation..."
            className={styles.searchInput}
            onClick={() => alert('Global search coming soon!')}
          />
        </div>

        <div className={styles.headerActions}>
          <a href="https://github.com/fiddupay/fiddupay-node" target="_blank" rel="noopener noreferrer" className={styles.headerLink}>GitHub</a>
          <a href="https://dashboard.fiddupay.com" className={styles.headerLink}>Dashboard</a>
        </div>
      </header>

      {/* Mobile Header (Visible only on mobile via CSS) */}
      <div className={mobileStyles.mobileHeader}>
        <div className={mobileStyles.logoSection}>
          <div className={mobileStyles.burgerIcon}>
            <div className={mobileStyles.bar}></div>
            <div className={mobileStyles.bar}></div>
            <div className={mobileStyles.bar}></div>
          </div>
          <Link to="/" className={mobileStyles.logoText}>FidduPay</Link>
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

      {/* Main Body Wrapper */}
      <div className={styles.docsBody}>

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
            let item = API_DATA.flatMap(s => [s, ...s.endpoints]).find(e => e.id === activeSection);

            // Fallback: If item is a Section (no request) but has endpoints, show the first endpoint's code
            let hasCode = false;
            let displayItem: Endpoint | undefined;

            if (item) {
              if ((item as any).request) {
                hasCode = true;
                displayItem = item as Endpoint;
              } else if ((item as any).endpoints && (item as any).endpoints.length > 0) {
                // It's a section with children
                const firstChild = (item as any).endpoints[0];
                if (firstChild.request) {
                  hasCode = true;
                  displayItem = firstChild;
                }
              }
            }

            return (
              <aside
                className={`${styles.codeColumn} ${mobileStyles.codeColumn}`}
                style={{ display: hasCode ? 'block' : 'none' }}
              >
                <div className={`${styles.codeSticky} ${mobileStyles.codeSticky}`}>
                  {hasCode && displayItem && (
                    <div key={displayItem.id || activeSection}>
                      <CodeSnippet
                        request={displayItem.request}
                        response={displayItem.response}
                        method={displayItem.method}
                        path={displayItem.path}
                      />
                    </div>
                  )}
                </div>
              </aside>
            );
          })()}
        </div>
      </div>
    </div>
  );
};

export default DocsPage;
