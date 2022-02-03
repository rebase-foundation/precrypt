import { useRouter } from 'next/router';

interface Flags {
  // devnet might be devdead
  network: 'mainnet-beta' | 'testnet';
}

export function useFlag<
  K extends keyof Flags,
  P extends Flags[K],
  Q extends Flags[K]
>(flag: K, defaultValue: P): Q {
  const router = useRouter();

  return router.query[flag] || (defaultValue as any);
}
