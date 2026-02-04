import { createIcons, Github, Layers, Globe, Target, Server, History, Bot } from 'lucide';

// Initialize Lucide icons
createIcons({
  icons: {
    Github,
    Layers,
    Globe,
    Target,
    Server,
    History,
    Bot
  }
});

// Staggered Animation for feature cards using Intersection Observer
const observerOptions = {
  threshold: 0.1
};

const observer = new IntersectionObserver((entries) => {
  entries.forEach((entry) => {
    if (entry.isIntersecting) {
      entry.target.classList.add('animate-fade-in-up');
      observer.unobserve(entry.target);
    }
  });
}, observerOptions);

document.querySelectorAll('.feature-card, .tech-tag, .step-node').forEach((el) => {
  el.style.opacity = '0'; // Initial state for observer
  observer.observe(el);
});
