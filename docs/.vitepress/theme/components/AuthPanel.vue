<template>
  <div class="auth-panel" aria-hidden="true">
    <div class="auth-panel__halo" />

    <div class="auth-panel__card">
      <div class="auth-panel__topbar">
        <span class="auth-panel__lock">
          <svg viewBox="0 0 16 16" fill="none">
            <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.2" />
            <path d="M5.5 7V5.5a2.5 2.5 0 0 1 5 0V7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </span>
        <span class="auth-panel__bus">org.freedesktop.PolicyKit1</span>
        <span class="auth-panel__status" />
      </div>

      <div class="auth-panel__body">
        <p class="auth-panel__eyebrow">Authorization request</p>
        <p class="auth-panel__title">Authentication required</p>
        <p class="auth-panel__action">org.freedesktop.policykit.exec</p>

        <div class="auth-panel__field">
          <span class="auth-panel__label">Password</span>
          <span class="auth-panel__dots">
            <i v-for="n in 8" :key="n" />
          </span>
          <span class="auth-panel__caret" />
        </div>

        <div class="auth-panel__actions">
          <span class="auth-panel__button auth-panel__button--ghost">Cancel</span>
          <span class="auth-panel__button auth-panel__button--primary">Authenticate</span>
        </div>
      </div>

      <div class="auth-panel__footer">
        <span class="auth-panel__footer-status" />
        <span>identity: unix-user</span>
        <span class="auth-panel__attempt">attempt 1 of 3</span>
      </div>
    </div>

    <div class="auth-panel__keyhole">
      <svg viewBox="0 0 120 120" fill="none">
        <circle cx="60" cy="46" r="22" stroke="currentColor" stroke-width="5" />
        <path d="M60 68v30" stroke="currentColor" stroke-width="5" stroke-linecap="round" />
        <circle cx="60" cy="46" r="6" fill="currentColor" />
      </svg>
    </div>

    <p class="auth-panel__caption">one dialog · no policy changes</p>
  </div>
</template>

<style scoped>
.auth-panel {
  position: relative;
  width: 100%;
  max-width: 420px;
  margin: 0 auto;
  padding: 28px 0;
}

.auth-panel__halo {
  position: absolute;
  inset: 4% 0;
  background: radial-gradient(circle at 50% 40%, var(--vp-c-brand-soft), transparent 68%);
  filter: blur(28px);
}

.auth-panel__card {
  position: relative;
  z-index: 2;
  overflow: hidden;
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  background: var(--vp-c-bg-elv);
  box-shadow: var(--iguard-shadow);
  transform: rotate(-1.5deg);
}

.auth-panel__topbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
}

.auth-panel__lock {
  display: inline-flex;
  width: 16px;
  height: 16px;
  color: var(--vp-c-brand-1);
}

.auth-panel__bus {
  flex: 1;
  overflow: hidden;
  color: var(--vp-c-text-3);
  font-family: var(--iguard-font-mono);
  font-size: 9px;
  letter-spacing: 0.04em;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.auth-panel__status {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--vp-c-tip-1);
  box-shadow: 0 0 0 3px var(--vp-c-tip-soft);
}

.auth-panel__body {
  padding: 22px 22px 20px;
}

.auth-panel__eyebrow {
  margin: 0 0 8px;
  color: var(--vp-c-brand-1);
  font-family: var(--iguard-font-mono);
  font-size: 9px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.auth-panel__title {
  margin: 0;
  color: var(--vp-c-text-1);
  font-family: var(--iguard-font-display);
  font-size: 23px;
  letter-spacing: -0.03em;
  line-height: 1.1;
}

.auth-panel__action {
  overflow: hidden;
  margin: 12px 0 18px;
  padding: 7px 9px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 5px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-2);
  font-family: var(--iguard-font-mono);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.auth-panel__field {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 7px;
  background: var(--vp-c-bg);
}

.auth-panel__label {
  color: var(--vp-c-text-3);
  font-family: var(--iguard-font-mono);
  font-size: 9px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.auth-panel__dots {
  display: flex;
  flex: 1;
  gap: 4px;
}

.auth-panel__dots i {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--vp-c-text-2);
  opacity: 0.65;
}

.auth-panel__caret {
  width: 1px;
  height: 14px;
  background: var(--vp-c-brand-1);
  animation: iguard-caret 1.15s steps(1) infinite;
}

.auth-panel__actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
}

.auth-panel__button {
  padding: 7px 12px;
  border-radius: 5px;
  font-family: var(--iguard-font-mono);
  font-size: 10px;
  letter-spacing: 0.03em;
}

.auth-panel__button--ghost {
  border: 1px solid var(--vp-c-divider);
  color: var(--vp-c-text-2);
}

.auth-panel__button--primary {
  background: var(--vp-c-brand-3);
  color: #0b1017;
  font-weight: 600;
}

.auth-panel__footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-top: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-3);
  font-family: var(--iguard-font-mono);
  font-size: 9px;
}

.auth-panel__footer-status {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--vp-c-tip-1);
  animation: iguard-pulse 2.4s ease-out infinite;
}

.auth-panel__attempt {
  margin-left: auto;
}

.auth-panel__keyhole {
  position: absolute;
  top: 50%;
  right: -30px;
  z-index: 1;
  width: 210px;
  height: 210px;
  color: var(--vp-c-brand-1);
  opacity: 0.1;
  transform: translateY(-50%) rotate(8deg);
}

.auth-panel__keyhole svg {
  width: 100%;
  height: 100%;
}

.auth-panel__caption {
  position: relative;
  z-index: 2;
  margin: 18px 0 0;
  color: var(--vp-c-text-3);
  font-family: var(--iguard-font-mono);
  font-size: 9px;
  letter-spacing: 0.12em;
  text-align: center;
  text-transform: uppercase;
}

@keyframes iguard-caret {
  0%,
  49% {
    opacity: 1;
  }

  50%,
  100% {
    opacity: 0;
  }
}

@keyframes iguard-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 0 var(--vp-c-tip-soft);
  }

  50% {
    box-shadow: 0 0 0 5px transparent;
  }
}

@media (max-width: 959px) {
  .auth-panel {
    max-width: 380px;
    padding: 8px 0 0;
  }

  .auth-panel__card {
    transform: none;
  }

  .auth-panel__keyhole {
    right: -48px;
    opacity: 0.08;
  }
}

@media (prefers-reduced-motion: reduce) {
  .auth-panel__caret,
  .auth-panel__footer-status {
    animation: none;
  }
}
</style>
