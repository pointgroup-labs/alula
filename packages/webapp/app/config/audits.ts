import halbornLogo from '~/assets/img/auditor/halborn.webp'
import highlandLogo from '~/assets/img/auditor/highland.png'

// `status` discriminates how the transparency page renders an auditor entry:
//   - 'completed' → show the audit date and "Read report" link
//   - 'pending'   → show "Coming soon" and disable the link
// `link` and `auditedAt` are required only for completed audits; pending
// entries can omit both, since there is nothing to read or date yet.
export type Auditor
  = | {
    status: 'completed'
    name: string
    logo: string
    link: string
    auditedAt: string
  }
  | {
    status: 'pending'
    name: string
    logo: string
  }

export const HALBORN_AUDITOR: Auditor = {
  status: 'completed',
  name: 'Halborn',
  link: 'https://www.halborn.com/audits/alula-finance/smart-contracts-cd8f6d',
  logo: halbornLogo,
  auditedAt: '2026-03-24',
}

export const HIGHLAND_SECURITY_AUDITOR: Auditor = {
  status: 'pending',
  name: 'Highland Security',
  logo: highlandLogo,
}

export const AUDITORS: Auditor[] = [
  HALBORN_AUDITOR,
  HIGHLAND_SECURITY_AUDITOR,
]
