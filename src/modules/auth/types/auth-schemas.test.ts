import { describe, expect, it } from 'vitest';

import { createUserSchema, loginSchema, passwordSchema, usernameSchema } from './auth-schemas';

describe('usernameSchema', () => {
  it('accepts a valid username', () => {
    expect(usernameSchema.safeParse('ada.lovelace_1').success).toBe(true);
  });

  it('rejects a username shorter than 3 characters', () => {
    expect(usernameSchema.safeParse('ab').success).toBe(false);
  });

  it('rejects a username with disallowed characters', () => {
    expect(usernameSchema.safeParse('ada lovelace!').success).toBe(false);
  });
});

describe('passwordSchema', () => {
  it('accepts a password meeting every rule', () => {
    expect(passwordSchema.safeParse('Sup3r-Secret-Pass').success).toBe(true);
  });

  it('rejects a password shorter than 12 characters', () => {
    expect(passwordSchema.safeParse('Sh0rt-P4ss').success).toBe(false);
  });

  it('rejects a password with no uppercase letter', () => {
    expect(passwordSchema.safeParse('sup3r-secret-pass').success).toBe(false);
  });

  it('rejects a password with no symbol', () => {
    expect(passwordSchema.safeParse('Sup3rSecretPass1').success).toBe(false);
  });
});

describe('createUserSchema', () => {
  it('accepts a fully valid user', () => {
    const result = createUserSchema.safeParse({
      fullName: 'Ada Lovelace',
      username: 'ada.lovelace',
      password: 'Sup3r-Secret-Pass',
      role: 'admin',
    });

    expect(result.success).toBe(true);
  });

  it('rejects an invalid role', () => {
    const result = createUserSchema.safeParse({
      fullName: 'Ada Lovelace',
      username: 'ada.lovelace',
      password: 'Sup3r-Secret-Pass',
      role: 'superadmin',
    });

    expect(result.success).toBe(false);
  });
});

describe('loginSchema', () => {
  it('requires a non-empty username and password', () => {
    expect(loginSchema.safeParse({ username: '', password: '' }).success).toBe(false);
    expect(loginSchema.safeParse({ username: 'ada', password: 'x' }).success).toBe(true);
  });
});
