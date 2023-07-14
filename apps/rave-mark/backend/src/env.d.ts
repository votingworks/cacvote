declare namespace NodeJS {
  export interface ProcessEnv {
    readonly CI?: string;
    readonly NODE_ENV: 'development' | 'production' | 'test';
    readonly IS_INTEGRATION_TEST?: string;
    readonly USE_MOCK_RAVE_SERVER?: string;
    readonly BASE_PORT?: string;
    readonly PORT?: string;
    readonly VX_MACHINE_ID?: string;
    readonly VX_CODE_VERSION?: string;
    readonly VX_SCREEN_ORIENTATION?: 'portrait' | 'landscape';
    readonly RAVE_MARK_WORKSPACE?: string;
  }
}
