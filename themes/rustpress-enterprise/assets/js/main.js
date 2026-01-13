/**
 * SaasNova Enterprise Theme - Main JavaScript
 * Core functionality and interactions
 */

(function() {
  'use strict';

  // ==========================================================================
  // DOM Ready
  // ==========================================================================
  document.addEventListener('DOMContentLoaded', init);

  function init() {
    initLoadingScreen();
    initHeader();
    initMobileMenu();
    initScrollProgress();
    initBackToTop();
    initCursorEffects();
    initSmoothScroll();
    initDropdowns();
    initFAQ();
    initCookieConsent();
    initForms();
    initCounters();
    initMagneticButtons();
    initRippleEffect();
    initParallax();
    initTabs();
    initTooltips();
  }

  // ==========================================================================
  // Loading Screen
  // ==========================================================================
  function initLoadingScreen() {
    const loadingScreen = document.getElementById('loading-screen');
    if (!loadingScreen) return;

    window.addEventListener('load', () => {
      setTimeout(() => {
        loadingScreen.classList.add('hidden');
        document.body.classList.add('loaded');

        // Remove loading screen after animation
        setTimeout(() => {
          loadingScreen.remove();
        }, 500);
      }, 500);
    });
  }

  // ==========================================================================
  // Header
  // ==========================================================================
  function initHeader() {
    const header = document.getElementById('site-header');
    if (!header) return;

    let lastScroll = 0;
    let ticking = false;

    function updateHeader() {
      const currentScroll = window.pageYOffset;

      // Add scrolled class
      if (currentScroll > 50) {
        header.classList.add('scrolled');
      } else {
        header.classList.remove('scrolled');
      }

      // Hide/show on scroll direction
      if (currentScroll > lastScroll && currentScroll > 200) {
        header.classList.add('hidden');
      } else {
        header.classList.remove('hidden');
      }

      lastScroll = currentScroll;
      ticking = false;
    }

    window.addEventListener('scroll', () => {
      if (!ticking) {
        requestAnimationFrame(updateHeader);
        ticking = true;
      }
    }, { passive: true });
  }

  // ==========================================================================
  // Mobile Menu
  // ==========================================================================
  function initMobileMenu() {
    const menuToggle = document.getElementById('menu-toggle');
    const mobileMenu = document.getElementById('mobile-menu');
    const mobileMenuClose = document.getElementById('mobile-menu-close');
    const mobileMenuBackdrop = mobileMenu?.querySelector('.mobile-menu-backdrop');
    const mobileNavToggles = document.querySelectorAll('.mobile-nav-toggle');

    if (!menuToggle || !mobileMenu) return;

    function openMenu() {
      menuToggle.classList.add('active');
      menuToggle.setAttribute('aria-expanded', 'true');
      mobileMenu.classList.add('active');
      mobileMenu.setAttribute('aria-hidden', 'false');
      document.body.style.overflow = 'hidden';
    }

    function closeMenu() {
      menuToggle.classList.remove('active');
      menuToggle.setAttribute('aria-expanded', 'false');
      mobileMenu.classList.remove('active');
      mobileMenu.setAttribute('aria-hidden', 'true');
      document.body.style.overflow = '';
    }

    menuToggle.addEventListener('click', () => {
      if (mobileMenu.classList.contains('active')) {
        closeMenu();
      } else {
        openMenu();
      }
    });

    mobileMenuClose?.addEventListener('click', closeMenu);
    mobileMenuBackdrop?.addEventListener('click', closeMenu);

    // Escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && mobileMenu.classList.contains('active')) {
        closeMenu();
      }
    });

    // Mobile submenu toggles
    mobileNavToggles.forEach(toggle => {
      toggle.addEventListener('click', () => {
        const isExpanded = toggle.getAttribute('aria-expanded') === 'true';

        // Close other submenus
        mobileNavToggles.forEach(t => {
          if (t !== toggle) {
            t.setAttribute('aria-expanded', 'false');
          }
        });

        toggle.setAttribute('aria-expanded', !isExpanded);
      });
    });

    // Close menu on link click
    const mobileLinks = mobileMenu.querySelectorAll('a');
    mobileLinks.forEach(link => {
      link.addEventListener('click', closeMenu);
    });
  }

  // ==========================================================================
  // Scroll Progress
  // ==========================================================================
  function initScrollProgress() {
    const progressBar = document.getElementById('scroll-progress');
    if (!progressBar) return;

    function updateProgress() {
      const scrollTop = window.pageYOffset;
      const docHeight = document.documentElement.scrollHeight - window.innerHeight;
      const progress = (scrollTop / docHeight) * 100;
      progressBar.style.transform = `scaleX(${progress / 100})`;
    }

    window.addEventListener('scroll', () => {
      requestAnimationFrame(updateProgress);
    }, { passive: true });
  }

  // ==========================================================================
  // Back to Top
  // ==========================================================================
  function initBackToTop() {
    const backToTop = document.getElementById('back-to-top');
    if (!backToTop) return;

    function toggleVisibility() {
      if (window.pageYOffset > 500) {
        backToTop.classList.add('visible');
      } else {
        backToTop.classList.remove('visible');
      }
    }

    window.addEventListener('scroll', () => {
      requestAnimationFrame(toggleVisibility);
    }, { passive: true });

    backToTop.addEventListener('click', () => {
      window.scrollTo({
        top: 0,
        behavior: 'smooth'
      });
    });
  }

  // ==========================================================================
  // Cursor Effects
  // ==========================================================================
  function initCursorEffects() {
    const cursorGlow = document.getElementById('cursor-glow');
    const cursorDot = document.getElementById('cursor-dot');

    if (!cursorGlow || !cursorDot) return;

    // Check for touch devices
    if ('ontouchstart' in window) return;

    let mouseX = 0;
    let mouseY = 0;
    let glowX = 0;
    let glowY = 0;

    document.addEventListener('mousemove', (e) => {
      mouseX = e.clientX;
      mouseY = e.clientY;

      cursorDot.style.left = `${mouseX}px`;
      cursorDot.style.top = `${mouseY}px`;
    });

    // Smooth follow for glow
    function animateGlow() {
      glowX += (mouseX - glowX) * 0.1;
      glowY += (mouseY - glowY) * 0.1;

      cursorGlow.style.left = `${glowX}px`;
      cursorGlow.style.top = `${glowY}px`;

      requestAnimationFrame(animateGlow);
    }
    animateGlow();

    // Show cursors on mouse enter
    document.addEventListener('mouseenter', () => {
      cursorGlow.classList.add('active');
      cursorDot.classList.add('active');
    });

    document.addEventListener('mouseleave', () => {
      cursorGlow.classList.remove('active');
      cursorDot.classList.remove('active');
    });

    // Hover effect on interactive elements
    const interactiveElements = document.querySelectorAll('a, button, input, textarea, select, [role="button"]');
    interactiveElements.forEach(el => {
      el.addEventListener('mouseenter', () => {
        cursorDot.classList.add('hover');
      });
      el.addEventListener('mouseleave', () => {
        cursorDot.classList.remove('hover');
      });
    });
  }

  // ==========================================================================
  // Smooth Scroll
  // ==========================================================================
  function initSmoothScroll() {
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
      anchor.addEventListener('click', function(e) {
        const href = this.getAttribute('href');
        if (href === '#' || href === '#0') return;

        const target = document.querySelector(href);
        if (target) {
          e.preventDefault();
          const headerHeight = document.getElementById('site-header')?.offsetHeight || 80;
          const targetPosition = target.offsetTop - headerHeight;

          window.scrollTo({
            top: targetPosition,
            behavior: 'smooth'
          });
        }
      });
    });
  }

  // ==========================================================================
  // Dropdowns
  // ==========================================================================
  function initDropdowns() {
    const dropdownTriggers = document.querySelectorAll('.nav-dropdown-trigger');

    dropdownTriggers.forEach(trigger => {
      trigger.addEventListener('click', (e) => {
        e.preventDefault();
        const dropdown = trigger.closest('.nav-dropdown');
        const isExpanded = trigger.getAttribute('aria-expanded') === 'true';

        // Close other dropdowns
        dropdownTriggers.forEach(t => {
          t.setAttribute('aria-expanded', 'false');
        });

        if (!isExpanded) {
          trigger.setAttribute('aria-expanded', 'true');
        }
      });
    });

    // Close on outside click
    document.addEventListener('click', (e) => {
      if (!e.target.closest('.nav-dropdown')) {
        dropdownTriggers.forEach(t => {
          t.setAttribute('aria-expanded', 'false');
        });
      }
    });
  }

  // ==========================================================================
  // FAQ Accordion
  // ==========================================================================
  function initFAQ() {
    const faqItems = document.querySelectorAll('.faq-item');

    faqItems.forEach(item => {
      const question = item.querySelector('.faq-question');

      question?.addEventListener('click', () => {
        const isActive = item.classList.contains('active');

        // Close all others
        faqItems.forEach(i => i.classList.remove('active'));

        // Toggle current
        if (!isActive) {
          item.classList.add('active');
        }
      });
    });
  }

  // ==========================================================================
  // Cookie Consent
  // ==========================================================================
  function initCookieConsent() {
    const cookieConsent = document.getElementById('cookie-consent');
    const acceptBtn = document.getElementById('cookie-accept');
    const declineBtn = document.getElementById('cookie-decline');

    if (!cookieConsent) return;

    // Check if user has already made a choice
    if (localStorage.getItem('cookie-consent')) {
      cookieConsent.remove();
      return;
    }

    // Show after delay
    setTimeout(() => {
      cookieConsent.classList.add('visible');
    }, 2000);

    acceptBtn?.addEventListener('click', () => {
      localStorage.setItem('cookie-consent', 'accepted');
      hideCookieConsent();
    });

    declineBtn?.addEventListener('click', () => {
      localStorage.setItem('cookie-consent', 'declined');
      hideCookieConsent();
    });

    function hideCookieConsent() {
      cookieConsent.classList.remove('visible');
      setTimeout(() => cookieConsent.remove(), 300);
    }
  }

  // ==========================================================================
  // Forms
  // ==========================================================================
  function initForms() {
    // Input focus effects
    const formInputs = document.querySelectorAll('.form-input, .form-textarea, .form-select');

    formInputs.forEach(input => {
      const formGroup = input.closest('.form-group');

      input.addEventListener('focus', () => {
        formGroup?.classList.add('focused');
      });

      input.addEventListener('blur', () => {
        formGroup?.classList.remove('focused');
        if (input.value) {
          formGroup?.classList.add('filled');
        } else {
          formGroup?.classList.remove('filled');
        }
      });
    });

    // Newsletter form
    const newsletterForms = document.querySelectorAll('.newsletter-form');
    newsletterForms.forEach(form => {
      form.addEventListener('submit', (e) => {
        e.preventDefault();
        const button = form.querySelector('button[type="submit"]');
        const originalText = button.innerHTML;

        button.innerHTML = '<span class="animate-spin">Subscribing...</span>';
        button.disabled = true;

        // Simulate API call
        setTimeout(() => {
          button.innerHTML = 'Subscribed!';
          button.style.background = 'var(--color-success)';
          form.reset();

          setTimeout(() => {
            button.innerHTML = originalText;
            button.style.background = '';
            button.disabled = false;
          }, 2000);
        }, 1500);
      });
    });
  }

  // ==========================================================================
  // Counters
  // ==========================================================================
  function initCounters() {
    const counters = document.querySelectorAll('[data-counter]');
    if (!counters.length) return;

    const observerOptions = {
      threshold: 0.5,
      rootMargin: '0px'
    };

    const counterObserver = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          animateCounter(entry.target);
          counterObserver.unobserve(entry.target);
        }
      });
    }, observerOptions);

    counters.forEach(counter => {
      counterObserver.observe(counter);
    });

    function animateCounter(element) {
      const target = parseInt(element.dataset.counter, 10);
      const suffix = element.dataset.suffix || '';
      const prefix = element.dataset.prefix || '';
      const duration = 2000;
      const startTime = performance.now();

      function update(currentTime) {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / duration, 1);

        // Easing function
        const easeOutQuart = 1 - Math.pow(1 - progress, 4);
        const currentValue = Math.floor(easeOutQuart * target);

        element.textContent = prefix + currentValue.toLocaleString() + suffix;

        if (progress < 1) {
          requestAnimationFrame(update);
        }
      }

      requestAnimationFrame(update);
    }
  }

  // ==========================================================================
  // Magnetic Buttons
  // ==========================================================================
  function initMagneticButtons() {
    const magneticElements = document.querySelectorAll('.magnetic, .btn--glow');

    if ('ontouchstart' in window) return;

    magneticElements.forEach(el => {
      el.addEventListener('mousemove', (e) => {
        const rect = el.getBoundingClientRect();
        const x = e.clientX - rect.left - rect.width / 2;
        const y = e.clientY - rect.top - rect.height / 2;

        el.style.transform = `translate(${x * 0.2}px, ${y * 0.2}px)`;
      });

      el.addEventListener('mouseleave', () => {
        el.style.transform = '';
      });
    });
  }

  // ==========================================================================
  // Ripple Effect
  // ==========================================================================
  function initRippleEffect() {
    const rippleElements = document.querySelectorAll('.btn--primary, .btn--secondary');

    rippleElements.forEach(el => {
      el.addEventListener('click', function(e) {
        const rect = this.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        const ripple = document.createElement('span');
        ripple.className = 'ripple';
        ripple.style.left = `${x}px`;
        ripple.style.top = `${y}px`;

        this.appendChild(ripple);

        setTimeout(() => ripple.remove(), 600);
      });
    });
  }

  // ==========================================================================
  // Parallax
  // ==========================================================================
  function initParallax() {
    const parallaxElements = document.querySelectorAll('[data-parallax]');
    if (!parallaxElements.length) return;

    function updateParallax() {
      const scrollY = window.pageYOffset;

      parallaxElements.forEach(el => {
        const speed = parseFloat(el.dataset.parallax) || 0.5;
        const rect = el.getBoundingClientRect();
        const centerY = rect.top + rect.height / 2;
        const offset = (centerY - window.innerHeight / 2) * speed;

        el.style.transform = `translateY(${offset}px)`;
      });
    }

    window.addEventListener('scroll', () => {
      requestAnimationFrame(updateParallax);
    }, { passive: true });
  }

  // ==========================================================================
  // Tabs
  // ==========================================================================
  function initTabs() {
    const tabContainers = document.querySelectorAll('[data-tabs]');

    tabContainers.forEach(container => {
      const tabs = container.querySelectorAll('[data-tab]');
      const panels = container.querySelectorAll('[data-panel]');

      tabs.forEach(tab => {
        tab.addEventListener('click', () => {
          const targetPanel = tab.dataset.tab;

          // Update tabs
          tabs.forEach(t => t.classList.remove('active'));
          tab.classList.add('active');

          // Update panels
          panels.forEach(panel => {
            if (panel.dataset.panel === targetPanel) {
              panel.classList.add('active');
            } else {
              panel.classList.remove('active');
            }
          });
        });
      });
    });
  }

  // ==========================================================================
  // Tooltips
  // ==========================================================================
  function initTooltips() {
    const tooltipElements = document.querySelectorAll('[data-tooltip]');

    tooltipElements.forEach(el => {
      const tooltipText = el.dataset.tooltip;

      el.addEventListener('mouseenter', () => {
        const tooltip = document.createElement('div');
        tooltip.className = 'tooltip';
        tooltip.textContent = tooltipText;
        document.body.appendChild(tooltip);

        const rect = el.getBoundingClientRect();
        tooltip.style.left = `${rect.left + rect.width / 2}px`;
        tooltip.style.top = `${rect.top - 10}px`;

        requestAnimationFrame(() => {
          tooltip.classList.add('visible');
        });

        el._tooltip = tooltip;
      });

      el.addEventListener('mouseleave', () => {
        if (el._tooltip) {
          el._tooltip.classList.remove('visible');
          setTimeout(() => {
            el._tooltip?.remove();
            el._tooltip = null;
          }, 200);
        }
      });
    });
  }

  // ==========================================================================
  // Utility Functions
  // ==========================================================================
  window.SaasNova = {
    // Debounce
    debounce(func, wait) {
      let timeout;
      return function executedFunction(...args) {
        const later = () => {
          clearTimeout(timeout);
          func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
      };
    },

    // Throttle
    throttle(func, limit) {
      let inThrottle;
      return function(...args) {
        if (!inThrottle) {
          func.apply(this, args);
          inThrottle = true;
          setTimeout(() => inThrottle = false, limit);
        }
      };
    },

    // Copy to clipboard
    async copyToClipboard(text) {
      try {
        await navigator.clipboard.writeText(text);
        return true;
      } catch (err) {
        console.error('Failed to copy:', err);
        return false;
      }
    },

    // Format number
    formatNumber(num) {
      return new Intl.NumberFormat().format(num);
    }
  };

})();
