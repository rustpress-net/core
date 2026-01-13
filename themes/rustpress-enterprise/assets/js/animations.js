/**
 * SaasNova Enterprise Theme - Animations
 * Scroll-triggered animations and effects
 */

(function() {
  'use strict';

  // ==========================================================================
  // Scroll Reveal Animations
  // ==========================================================================
  class ScrollReveal {
    constructor() {
      this.elements = document.querySelectorAll('[data-animate]');
      this.staggerElements = document.querySelectorAll('[data-stagger]');

      if (!this.elements.length && !this.staggerElements.length) return;

      this.observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
      };

      this.init();
    }

    init() {
      // Check for reduced motion preference
      if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
        this.showAllElements();
        return;
      }

      this.observer = new IntersectionObserver(
        this.handleIntersection.bind(this),
        this.observerOptions
      );

      this.elements.forEach(el => this.observer.observe(el));
      this.staggerElements.forEach(el => this.observer.observe(el));
    }

    handleIntersection(entries) {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          const delay = parseInt(entry.target.dataset.delay, 10) || 0;

          setTimeout(() => {
            entry.target.classList.add('animated');
          }, delay);

          this.observer.unobserve(entry.target);
        }
      });
    }

    showAllElements() {
      this.elements.forEach(el => {
        el.style.opacity = '1';
        el.style.transform = 'none';
      });
      this.staggerElements.forEach(el => {
        el.querySelectorAll('*').forEach(child => {
          child.style.opacity = '1';
          child.style.transform = 'none';
        });
      });
    }
  }

  // ==========================================================================
  // Text Split Animation
  // ==========================================================================
  class TextSplitter {
    constructor(selector) {
      this.elements = document.querySelectorAll(selector);
      this.init();
    }

    init() {
      this.elements.forEach(el => {
        const text = el.textContent;
        el.innerHTML = '';
        el.setAttribute('aria-label', text);

        const words = text.split(' ');
        words.forEach((word, wordIndex) => {
          const wordSpan = document.createElement('span');
          wordSpan.className = 'word';
          wordSpan.style.display = 'inline-block';
          wordSpan.style.marginRight = '0.25em';

          [...word].forEach((char, charIndex) => {
            const charSpan = document.createElement('span');
            charSpan.className = 'char';
            charSpan.textContent = char;
            charSpan.style.animationDelay = `${(wordIndex * 5 + charIndex) * 30}ms`;
            wordSpan.appendChild(charSpan);
          });

          el.appendChild(wordSpan);
        });
      });

      // Observe for animation trigger
      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            entry.target.classList.add('animated');
            observer.unobserve(entry.target);
          }
        });
      }, { threshold: 0.5 });

      this.elements.forEach(el => observer.observe(el));
    }
  }

  // ==========================================================================
  // Typewriter Effect
  // ==========================================================================
  class Typewriter {
    constructor(element, options = {}) {
      this.element = element;
      this.words = JSON.parse(element.dataset.typewriter || '[]');
      this.typingSpeed = options.typingSpeed || 100;
      this.deletingSpeed = options.deletingSpeed || 50;
      this.pauseTime = options.pauseTime || 2000;
      this.currentWordIndex = 0;
      this.currentCharIndex = 0;
      this.isDeleting = false;

      if (this.words.length > 0) {
        this.type();
      }
    }

    type() {
      const currentWord = this.words[this.currentWordIndex];

      if (this.isDeleting) {
        this.currentCharIndex--;
      } else {
        this.currentCharIndex++;
      }

      this.element.textContent = currentWord.substring(0, this.currentCharIndex);

      let delay = this.isDeleting ? this.deletingSpeed : this.typingSpeed;

      if (!this.isDeleting && this.currentCharIndex === currentWord.length) {
        delay = this.pauseTime;
        this.isDeleting = true;
      } else if (this.isDeleting && this.currentCharIndex === 0) {
        this.isDeleting = false;
        this.currentWordIndex = (this.currentWordIndex + 1) % this.words.length;
      }

      setTimeout(() => this.type(), delay);
    }
  }

  // ==========================================================================
  // 3D Tilt Effect
  // ==========================================================================
  class Tilt3D {
    constructor(selector) {
      this.elements = document.querySelectorAll(selector);

      if ('ontouchstart' in window) return;

      this.elements.forEach(el => {
        el.addEventListener('mousemove', this.handleMouseMove.bind(this, el));
        el.addEventListener('mouseleave', this.handleMouseLeave.bind(this, el));
      });
    }

    handleMouseMove(el, e) {
      const rect = el.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      const centerX = rect.width / 2;
      const centerY = rect.height / 2;

      const tiltX = (y - centerY) / centerY * -10;
      const tiltY = (x - centerX) / centerX * 10;

      el.style.setProperty('--tilt-x', `${tiltX}deg`);
      el.style.setProperty('--tilt-y', `${tiltY}deg`);

      const inner = el.querySelector('.tilt-3d-inner');
      if (inner) {
        inner.style.transform = `rotateX(${tiltX}deg) rotateY(${tiltY}deg)`;
      }
    }

    handleMouseLeave(el) {
      el.style.setProperty('--tilt-x', '0deg');
      el.style.setProperty('--tilt-y', '0deg');

      const inner = el.querySelector('.tilt-3d-inner');
      if (inner) {
        inner.style.transform = '';
      }
    }
  }

  // ==========================================================================
  // Morphing Background Shapes
  // ==========================================================================
  class MorphingShapes {
    constructor() {
      this.shapes = document.querySelectorAll('.morph-shape');
      this.animate();
    }

    animate() {
      this.shapes.forEach((shape, index) => {
        const baseDelay = index * 2;
        shape.style.animation = `morphBlob ${8 + index * 2}s ease-in-out ${baseDelay}s infinite`;
      });
    }
  }

  // ==========================================================================
  // Particle System
  // ==========================================================================
  class ParticleSystem {
    constructor(container, options = {}) {
      this.container = container;
      this.options = {
        particleCount: options.particleCount || 50,
        colors: options.colors || ['#6366F1', '#06B6D4', '#818CF8'],
        minSize: options.minSize || 2,
        maxSize: options.maxSize || 6,
        minSpeed: options.minSpeed || 0.5,
        maxSpeed: options.maxSpeed || 2,
        ...options
      };

      this.particles = [];
      this.canvas = document.createElement('canvas');
      this.ctx = this.canvas.getContext('2d');

      this.init();
    }

    init() {
      this.canvas.style.cssText = `
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
      `;
      this.container.appendChild(this.canvas);
      this.resize();

      window.addEventListener('resize', () => this.resize());

      this.createParticles();
      this.animate();
    }

    resize() {
      this.canvas.width = this.container.offsetWidth;
      this.canvas.height = this.container.offsetHeight;
    }

    createParticles() {
      for (let i = 0; i < this.options.particleCount; i++) {
        this.particles.push({
          x: Math.random() * this.canvas.width,
          y: Math.random() * this.canvas.height,
          size: this.random(this.options.minSize, this.options.maxSize),
          speedX: this.random(-this.options.maxSpeed, this.options.maxSpeed),
          speedY: this.random(-this.options.maxSpeed, this.options.maxSpeed),
          color: this.options.colors[Math.floor(Math.random() * this.options.colors.length)],
          opacity: Math.random() * 0.5 + 0.2
        });
      }
    }

    random(min, max) {
      return Math.random() * (max - min) + min;
    }

    animate() {
      this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

      this.particles.forEach(particle => {
        particle.x += particle.speedX;
        particle.y += particle.speedY;

        // Wrap around
        if (particle.x < 0) particle.x = this.canvas.width;
        if (particle.x > this.canvas.width) particle.x = 0;
        if (particle.y < 0) particle.y = this.canvas.height;
        if (particle.y > this.canvas.height) particle.y = 0;

        this.ctx.beginPath();
        this.ctx.arc(particle.x, particle.y, particle.size, 0, Math.PI * 2);
        this.ctx.fillStyle = particle.color;
        this.ctx.globalAlpha = particle.opacity;
        this.ctx.fill();
      });

      requestAnimationFrame(() => this.animate());
    }
  }

  // ==========================================================================
  // Marquee Scroll
  // ==========================================================================
  class MarqueeScroll {
    constructor(selector) {
      this.elements = document.querySelectorAll(selector);

      this.elements.forEach(el => {
        const content = el.innerHTML;
        el.innerHTML = content + content;

        const speed = parseFloat(el.dataset.speed) || 30;
        const direction = el.dataset.direction === 'right' ? 1 : -1;

        el.style.animation = `scrollLogos ${speed}s linear infinite`;
        if (direction === 1) {
          el.style.animationDirection = 'reverse';
        }
      });
    }
  }

  // ==========================================================================
  // Scroll-Linked Animations
  // ==========================================================================
  class ScrollLinked {
    constructor() {
      this.elements = document.querySelectorAll('[data-scroll-progress]');
      if (!this.elements.length) return;

      this.update();
      window.addEventListener('scroll', () => {
        requestAnimationFrame(() => this.update());
      }, { passive: true });
    }

    update() {
      const scrollY = window.pageYOffset;
      const windowHeight = window.innerHeight;
      const docHeight = document.documentElement.scrollHeight;

      this.elements.forEach(el => {
        const rect = el.getBoundingClientRect();
        const elementTop = rect.top + scrollY;
        const elementHeight = rect.height;

        // Calculate progress (0 to 1) as element scrolls through viewport
        const start = elementTop - windowHeight;
        const end = elementTop + elementHeight;
        const progress = Math.max(0, Math.min(1, (scrollY - start) / (end - start)));

        el.style.setProperty('--scroll-progress', progress);
      });
    }
  }

  // ==========================================================================
  // Number Counter Animation
  // ==========================================================================
  class NumberCounter {
    constructor(element, options = {}) {
      this.element = element;
      this.target = parseFloat(element.dataset.counter);
      this.prefix = element.dataset.prefix || '';
      this.suffix = element.dataset.suffix || '';
      this.decimals = parseInt(element.dataset.decimals, 10) || 0;
      this.duration = options.duration || 2000;
      this.animated = false;

      this.observe();
    }

    observe() {
      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting && !this.animated) {
            this.animate();
            this.animated = true;
            observer.unobserve(this.element);
          }
        });
      }, { threshold: 0.5 });

      observer.observe(this.element);
    }

    animate() {
      const startTime = performance.now();

      const update = (currentTime) => {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / this.duration, 1);

        // Easing
        const eased = 1 - Math.pow(1 - progress, 4);
        const current = eased * this.target;

        this.element.textContent = this.prefix +
          current.toFixed(this.decimals).replace(/\B(?=(\d{3})+(?!\d))/g, ',') +
          this.suffix;

        if (progress < 1) {
          requestAnimationFrame(update);
        }
      };

      requestAnimationFrame(update);
    }
  }

  // ==========================================================================
  // Image Reveal Effect
  // ==========================================================================
  class ImageReveal {
    constructor(selector) {
      this.images = document.querySelectorAll(selector);

      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            entry.target.classList.add('revealed');
            observer.unobserve(entry.target);
          }
        });
      }, { threshold: 0.3 });

      this.images.forEach(img => observer.observe(img));
    }
  }

  // ==========================================================================
  // Floating Elements
  // ==========================================================================
  class FloatingElements {
    constructor(selector) {
      this.elements = document.querySelectorAll(selector);

      this.elements.forEach((el, index) => {
        const delay = index * 0.5;
        const duration = 4 + Math.random() * 2;

        el.style.animation = `float ${duration}s ease-in-out ${delay}s infinite`;
      });
    }
  }

  // ==========================================================================
  // Glow Follow Cursor
  // ==========================================================================
  class GlowFollow {
    constructor(selector) {
      this.cards = document.querySelectorAll(selector);

      if ('ontouchstart' in window) return;

      this.cards.forEach(card => {
        card.addEventListener('mousemove', (e) => {
          const rect = card.getBoundingClientRect();
          const x = e.clientX - rect.left;
          const y = e.clientY - rect.top;

          card.style.setProperty('--mouse-x', `${x}px`);
          card.style.setProperty('--mouse-y', `${y}px`);
        });
      });
    }
  }

  // ==========================================================================
  // Initialize All Animations
  // ==========================================================================
  document.addEventListener('DOMContentLoaded', () => {
    // Core animations
    new ScrollReveal();
    new ScrollLinked();

    // Text animations
    new TextSplitter('.split-text');

    // Typewriter effects
    document.querySelectorAll('[data-typewriter]').forEach(el => {
      new Typewriter(el);
    });

    // 3D effects
    new Tilt3D('.tilt-3d');

    // Floating elements
    new FloatingElements('.float');

    // Marquee
    new MarqueeScroll('.logo-scroll-inner');

    // Counter animations
    document.querySelectorAll('[data-counter]').forEach(el => {
      new NumberCounter(el);
    });

    // Image reveals
    new ImageReveal('[data-reveal]');

    // Glow follow
    new GlowFollow('.feature-card, .pricing-card');

    // Particle system for hero sections
    const heroParticles = document.querySelector('.hero-particles');
    if (heroParticles) {
      new ParticleSystem(heroParticles, {
        particleCount: 30,
        minSize: 1,
        maxSize: 3,
        minSpeed: 0.2,
        maxSpeed: 0.8
      });
    }
  });

  // Export for external use
  window.SaasNovaAnimations = {
    ScrollReveal,
    TextSplitter,
    Typewriter,
    Tilt3D,
    ParticleSystem,
    NumberCounter,
    MarqueeScroll
  };

})();
