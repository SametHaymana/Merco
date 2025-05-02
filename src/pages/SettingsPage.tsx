import React, { useState, useEffect } from 'react';
import './SettingsPage.css'; // We'll create this CSS file next

// Define LLM Configuration Structure
interface LLMConfig {
  provider: 'ollama' | 'openrouter'; // Keep provider types specific
  apiKey: string;
  baseUrl: string;
  modelName: string;
}

// Define Application Behavior Config Structure
interface AppBehaviorConfig {
  autoSaveSettings: boolean;
  defaultPage: '/' | '/settings'; // Add more page routes as needed
}

// Template Configurations
const ollamaTemplate: LLMConfig = {
  provider: 'ollama',
  apiKey: '', // Ollama typically doesn't need one
  baseUrl: 'http://localhost:11434', // Common default
  modelName: 'llama3:latest', // Example model
};

const openRouterTemplate: LLMConfig = {
  provider: 'openrouter',
  apiKey: '', // User needs to provide
  baseUrl: 'https://openrouter.ai/api/v1', // Default OpenRouter URL
  modelName: 'openai/gpt-4o', // Example model
};

// --- Appearance Settings --- //

// Define Appearance Configuration Structure
interface AppearanceConfig {
  primaryColor: string;
  secondaryColor: string;
  backgroundColor: string;
  textColor: string;
  fontFamily: string;
}

// Default Appearance (matches initial theme)
const defaultAppearance: AppearanceConfig = {
  primaryColor: '#daa520',
  secondaryColor: '#00ffff',
  backgroundColor: '#1a1a1a',
  textColor: '#e0e0e0',
  fontFamily: 'Orbitron', // Make sure Orbitron is loaded via App.css or other means
};

// Default Behavior
const defaultBehavior: AppBehaviorConfig = {
  autoSaveSettings: true,
  defaultPage: '/',
};

// Define setting categories (can be expanded later)
type SettingsCategory = 'llm' | 'appearance' | 'behavior';

const SettingsPage: React.FC = () => {
  const [activeCategory, setActiveCategory] = useState<SettingsCategory>('llm');
  const [llmProvider, setLlmProvider] = useState<LLMConfig['provider']>(ollamaTemplate.provider);
  const [apiKey, setApiKey] = useState<string>(ollamaTemplate.apiKey);
  const [baseUrl, setBaseUrl] = useState<string>(ollamaTemplate.baseUrl);
  const [modelName, setModelName] = useState<string>(ollamaTemplate.modelName);
  const [ollamaMode, setOllamaMode] = useState<'existing' | 'create'>('existing'); // New state for Ollama mode

  // Appearance State
  const [appearanceConfig, setAppearanceConfig] = useState<AppearanceConfig>(defaultAppearance);

  // Behavior State
  const [appBehaviorConfig, setAppBehaviorConfig] = useState<AppBehaviorConfig>(defaultBehavior);

  // Function to load a template
  const loadTemplate = (template: LLMConfig) => {
    setLlmProvider(template.provider);
    setApiKey(template.apiKey);
    setBaseUrl(template.baseUrl);
    setModelName(template.modelName);
    // Reset ollamaMode when loading template (implies 'existing' initially)
    if (template.provider === 'ollama') {
      setOllamaMode('existing');
    }
  };

  // Updated handler for provider change
  const handleProviderChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const newProvider = event.target.value as LLMConfig['provider'];
    if (newProvider === 'ollama') {
      loadTemplate(ollamaTemplate); // This also resets ollamaMode via loadTemplate
    } else if (newProvider === 'openrouter') {
      loadTemplate(openRouterTemplate);
    }
  };

  const handleOllamaModeChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setOllamaMode(event.target.value as 'existing' | 'create');
    // Optionally clear/reset URL/Model when switching to 'create'?
    // if (event.target.value === 'create') {
    //    setBaseUrl('');
    //    setModelName('');
    // }
  };

  // Function to apply appearance settings to CSS variables
  const applyAppearanceSettings = (config: AppearanceConfig) => {
    const root = document.documentElement;
    root.style.setProperty('--primary-gold', config.primaryColor);
    root.style.setProperty('--secondary-cyan', config.secondaryColor);
    root.style.setProperty('--dark-bg', config.backgroundColor);
    root.style.setProperty('--light-text', config.textColor);
    root.style.setProperty('--primary-font', config.fontFamily + ', sans-serif');
    // Basic border adjustment based on background lightness
    const bgIsDark = parseInt(config.backgroundColor.substring(1, 3), 16) < 128; // Simple dark check
    root.style.setProperty('--border-color', bgIsDark ? '#444444' : '#cccccc');
    // TODO: More sophisticated derived colors (hover, etc.) could be added
  };

  // Apply settings on mount and when config changes
  useEffect(() => {
    applyAppearanceSettings(appearanceConfig);
  }, [appearanceConfig]);

  // Handler for appearance changes
  const handleAppearanceChange = (key: keyof AppearanceConfig, value: string) => {
    setAppearanceConfig(prevConfig => ({
      ...prevConfig,
      [key]: value,
    }));
  };

  // Handler for behavior changes
  const handleAppBehaviorChange = (key: keyof AppBehaviorConfig, value: string | boolean) => {
    setAppBehaviorConfig(prevConfig => ({
      ...prevConfig,
      [key]: value,
    }));
    // TODO: Implement actual save logic if autoSaveSettings is true
    // if (key !== 'autoSaveSettings' && appBehaviorConfig.autoSaveSettings) {
    //   console.log("Attempting to auto-save behavior change...", { [key]: value });
    //   // saveSettings(); 
    // }
  };

  const renderSettingsContent = () => {
    // Helper to get provider display name
    const getProviderDisplayName = (provider: string) => {
      switch (provider) {
        case 'ollama': return 'Ollama';
        case 'openrouter': return 'OpenRouter';
        default: return 'Unknown Provider';
      }
    };

    const showOllamaAdvanced = llmProvider === 'ollama' && ollamaMode === 'existing';
    const showOpenRouterFields = llmProvider === 'openrouter';

    switch (activeCategory) {
      case 'llm':
        return (
          <div className="settings-content-section">
            <h2>LLM Configuration</h2>
            <p>Select a provider and configure its settings.</p>

            {/* Provider Selection */}
            <div className="form-group">
              <label htmlFor="llm-provider">LLM Provider:</label>
              <select
                id="llm-provider"
                value={llmProvider}
                onChange={handleProviderChange}
              >
                <option value="ollama">Ollama</option>
                <option value="openrouter">OpenRouter</option>
              </select>
            </div>

            {/* Ollama Mode Selection (Conditional) */}
            {llmProvider === 'ollama' && (
              <div className="form-group radio-group">
                <label>Ollama Setup:</label>
                <div className="radio-options">
                  <label className="radio-option">
                    <input
                      type="radio"
                      name="ollamaMode"
                      value="existing"
                      checked={ollamaMode === 'existing'}
                      onChange={handleOllamaModeChange}
                    />
                    <span>Use Existing Ollama Service</span>
                  </label>
                  <label className="radio-option">
                    <input
                      type="radio"
                      name="ollamaMode"
                      value="create"
                      checked={ollamaMode === 'create'}
                      onChange={handleOllamaModeChange}
                    />
                    <span>Create & Manage New Engine (Experimental)</span>
                  </label>
                </div>
              </div>
            )}

            {/* API Key Input (Conditional based on provider needs) */}
            {/* Hide for Ollama create mode? */}
            {(showOpenRouterFields || (llmProvider === 'ollama' && ollamaMode === 'existing' /* Or always show if might be needed? */)) && (
              <div className="form-group">
                <label htmlFor="llm-api-key">API Key (if required):</label>
                <input
                  type="password"
                  id="llm-api-key"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder={`Enter API key for ${getProviderDisplayName(llmProvider)} (if applicable)`}
                  disabled={llmProvider === 'ollama' && ollamaMode === 'create'} // Example: Disable for create mode
                />
              </div>
            )}

            {/* Base URL Input (Conditional) */}
            {(showOllamaAdvanced || showOpenRouterFields) && (
              <div className="form-group">
                <label htmlFor="llm-base-url">Base URL:</label>
                <input
                  type="text"
                  id="llm-base-url"
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  placeholder={llmProvider === 'ollama' ? "e.g., http://localhost:11434" : "e.g., https://api.openrouter.ai/v1"}
                />
              </div>
            )}

            {/* Model Name Input (Conditional) */}
            {(showOllamaAdvanced || showOpenRouterFields) && (
              <div className="form-group">
                <label htmlFor="llm-model-name">Model Name:</label>
                <input
                  type="text"
                  id="llm-model-name"
                  value={modelName}
                  onChange={(e) => setModelName(e.target.value)}
                  placeholder={llmProvider === 'ollama' ? "e.g., llama3:latest" : "e.g., openai/gpt-4o"}
                />
              </div>
            )}
          </div>
        );
      case 'appearance':
        return (
            <div className="settings-content-section">
                <h2>Appearance</h2>
                <p>Customize the application's look and feel.</p>

                <div className="form-grid"> {/* Use a grid */}                   
                    <div className="form-group">
                        <label htmlFor="primary-color">Primary Accent:</label>
                        <input
                            type="color"
                            id="primary-color"
                            value={appearanceConfig.primaryColor}
                            onChange={(e) => handleAppearanceChange('primaryColor', e.target.value)}
                        />
                    </div>
                    <div className="form-group">
                        <label htmlFor="secondary-color">Secondary Accent:</label>
                        <input
                            type="color"
                            id="secondary-color"
                            value={appearanceConfig.secondaryColor}
                            onChange={(e) => handleAppearanceChange('secondaryColor', e.target.value)}
                        />
                    </div>
                    <div className="form-group">
                        <label htmlFor="background-color">Background:</label>
                        <input
                            type="color"
                            id="background-color"
                            value={appearanceConfig.backgroundColor}
                            onChange={(e) => handleAppearanceChange('backgroundColor', e.target.value)}
                        />
                    </div>
                    <div className="form-group">
                        <label htmlFor="text-color">Text:</label>
                        <input
                            type="color"
                            id="text-color"
                            value={appearanceConfig.textColor}
                            onChange={(e) => handleAppearanceChange('textColor', e.target.value)}
                        />
                    </div>
                </div>

                 <div className="form-group">
                     <label htmlFor="font-family">Font Family:</label>
                     <select
                         id="font-family"
                         value={appearanceConfig.fontFamily}
                         onChange={(e) => handleAppearanceChange('fontFamily', e.target.value)}
                     >
                         <option value="Orbitron">Orbitron (Futuristic)</option>
                         <option value="Inter">Inter (Modern Sans)</option>
                         <option value="Roboto">Roboto (Google Sans)</option>
                         <option value="system-ui">System Default</option>
                     </select>
                 </div>

                 <div style={{ marginTop: '1.5rem' }}>
                     <button
                       className="button-secondary"
                       onClick={() => setAppearanceConfig(defaultAppearance)}
                     >
                         Reset to Defaults
                     </button>
                 </div>
            </div>
        );
      case 'behavior':
        return (
          <div className="settings-content-section">
            <h2>Application Behavior</h2>
            <p>Control how the application behaves.</p>

            {/* Auto-Save Toggle */}
            <div className="form-group checkbox-group">
                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={appBehaviorConfig.autoSaveSettings}
                        onChange={(e) => handleAppBehaviorChange('autoSaveSettings', e.target.checked)}
                    />
                    <span>Automatically Save Settings Changes</span>
                </label>
                <p className="setting-description">
                    When enabled, changes to settings will be saved automatically. Otherwise, a manual save action might be required (not yet implemented).
                </p>
            </div>

            {/* Default Page Select */}
            <div className="form-group">
              <label htmlFor="default-page">Default Page on Startup:</label>
              <select
                id="default-page"
                value={appBehaviorConfig.defaultPage}
                onChange={(e) => handleAppBehaviorChange('defaultPage', e.target.value as AppBehaviorConfig['defaultPage'])}
              >
                <option value="/">Home Page</option>
                <option value="/settings">Settings Page</option>
              </select>
               <p className="setting-description">
                    Choose which page the application should open to by default.
                </p>
            </div>

            {/* Reset Button */}
             <div style={{ marginTop: '1.5rem' }}>
                 <button
                   className="button-secondary"
                   onClick={() => setAppBehaviorConfig(defaultBehavior)}
                 >
                     Reset Behavior Defaults
                 </button>
             </div>

          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="settings-page-layout"> {/* Overall container for the new layout */}
      <h1 className="settings-page-title">Settings</h1>
      <div className="settings-main-content"> {/* Container for nav + content */}
        <nav className="settings-nav">
          <ul>
            <li>
              <button
                className={activeCategory === 'llm' ? 'active' : ''}
                onClick={() => setActiveCategory('llm')}
              >
                LLM Configuration
              </button>
            </li>
             <li>
              <button
                className={activeCategory === 'appearance' ? 'active' : ''}
                onClick={() => setActiveCategory('appearance')}
              >
                Appearance
              </button>
            </li>
            <li>
              <button
                className={activeCategory === 'behavior' ? 'active' : ''}
                onClick={() => setActiveCategory('behavior')}
              >
                Application Behavior
              </button>
            </li>
          </ul>
        </nav>

        <div className="settings-content">
          {renderSettingsContent()}
        </div>
      </div>
    </div>
  );
};

export default SettingsPage; 