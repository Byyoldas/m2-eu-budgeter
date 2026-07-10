/**
 * Generic form field wrapper with label, input, and error message.
 * Renders an accessible label + input + optional hint + error.
 */

import React from 'react';

interface FormFieldProps {
  label: string;
  htmlFor: string;
  error?: string;
  hint?: string;
  required?: boolean;
  children: React.ReactNode;
}

export function FormField({ label, htmlFor, error, hint, required, children }: FormFieldProps) {
  return (
    <div className="form-field">
      <label htmlFor={htmlFor} className={`form-label${required ? ' required' : ''}`}>
        {label}
      </label>
      {children}
      {hint && !error && <span className="form-hint">{hint}</span>}
      {error && <span className="form-error" role="alert">{error}</span>}
    </div>
  );
}

interface SelectFieldProps {
  label: string;
  htmlFor: string;
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string }[];
  error?: string;
  hint?: string;
  required?: boolean;
  placeholder?: string;
}

export function SelectField({
  label, htmlFor, value, onChange, options, error, hint, required, placeholder,
}: SelectFieldProps) {
  return (
    <FormField label={label} htmlFor={htmlFor} error={error} hint={hint} required={required}>
      <select
        id={htmlFor}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={`form-input${error ? ' form-input--error' : ''}`}
      >
        {placeholder && <option value="">{placeholder}</option>}
        {options.map((o) => (
          <option key={o.value} value={o.value}>{o.label}</option>
        ))}
      </select>
    </FormField>
  );
}

interface NumberInputProps {
  label: string;
  htmlFor: string;
  value: string;
  onChange: (v: string) => void;
  error?: string;
  hint?: string;
  required?: boolean;
  min?: number;
  max?: number;
  step?: number;
  placeholder?: string;
}

export function NumberInput({
  label, htmlFor, value, onChange, error, hint, required, min, max, step, placeholder,
}: NumberInputProps) {
  return (
    <FormField label={label} htmlFor={htmlFor} error={error} hint={hint} required={required}>
      <input
        type="number"
        id={htmlFor}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        min={min}
        max={max}
        step={step ?? 'any'}
        placeholder={placeholder}
        className={`form-input${error ? ' form-input--error' : ''}`}
      />
    </FormField>
  );
}

interface TextInputProps {
  label: string;
  htmlFor: string;
  value: string;
  onChange: (v: string) => void;
  error?: string;
  hint?: string;
  required?: boolean;
  placeholder?: string;
  type?: 'text' | 'date';
}

export function TextInput({
  label, htmlFor, value, onChange, error, hint, required, placeholder, type = 'text',
}: TextInputProps) {
  return (
    <FormField label={label} htmlFor={htmlFor} error={error} hint={hint} required={required}>
      <input
        type={type}
        id={htmlFor}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className={`form-input${error ? ' form-input--error' : ''}`}
      />
    </FormField>
  );
}
