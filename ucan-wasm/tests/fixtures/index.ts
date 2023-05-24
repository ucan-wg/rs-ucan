import invalid from './invalid.json'
import valid from './valid.json'

type Expectation = 'valid' | 'invalid'

type Fixture = {
  comment: string
  token: string
  assertions: {
    header: {
      alg: "EdDSA"
      typ: "JWT"
      ucv: "0.9.0-canary"
    },
    payload: {
      iss: string
      aud: string
      exp: number | null
      nbf?: number
      nnc?: string
      fct: Record<string,unknown>[],
      att: {with: string, can: string}[],
      prf: string[]
    },
    validationErrors?: string[]
  }

}

export function getFixture(expectation: Expectation, comment: string): Fixture {
  let fixture

  if (expectation === 'valid') {
    fixture = valid.find(f => f.comment === comment)
  } else if (expectation === 'invalid') {
    fixture = invalid.find(f => f.comment === comment)
  }

  return fixture
}
