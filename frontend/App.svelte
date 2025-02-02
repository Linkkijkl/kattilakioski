<script lang="ts">
    import IconButton, { Icon } from '@smui/icon-button';
    import TopAppBar, { Row, Section, Title as TopAppBarTitle } from '@smui/top-app-bar';
    import { mdiMenu, mdiSearchWeb, mdiBasket, mdiCash, mdiAccountCircle } from '@mdi/js';
    import Drawer, {
      AppContent,
      Content,
      Header,
      Title as DrawerTitle,
      Subtitle,
      Scrim,
  } from '@smui/drawer';
  import List, { Item, Text, Graphic, Separator, Subheader } from '@smui/list';
  
  let drawerOpen = $state(false);
  let view = $state('buy');
  const setView = (val: string) => view = val; 
</script>

<main>
  <Drawer variant="modal" bind:open={drawerOpen}>
    <Header>
      <DrawerTitle>Kattilakioski</DrawerTitle>
      <Subtitle></Subtitle>
    </Header>
    <Content>
      <List>
        <Item
          href="javascript:void(0)"
          onclick={() => setView('buy')}
          activated={view === 'buy'}
        >
          <IconButton aria-label="Buy">
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiBasket} />
            </Icon>
          </IconButton>
          <Text>Buy</Text>
        </Item>
        <Item
          href="javascript:void(0)"
          onclick={() => setView('sell')}
          activated={view === 'sell'}
        >
        <IconButton aria-label="Sell">
          <Icon tag="svg" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiCash} />
          </Icon>
        </IconButton>
          <Text>Sell</Text>
        </Item>
        <Item
          href="javascript:void(0)"
          onclick={() => setView('login')}
          activated={view === 'login'}
        >
          <IconButton aria-label="Login">
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiAccountCircle} />
            </Icon>
          </IconButton>
          <Text>Login</Text>
        </Item>
      </List>
    </Content>
  </Drawer>
  <AppContent class="flexor-content">
    <TopAppBar
      variant="static"
      color='primary'
    >
      <Row>
        <Section>
          <IconButton onclick={() => (drawerOpen = !drawerOpen)}>
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiMenu} />
            </Icon>
          </IconButton>
          <TopAppBarTitle>Kattilakioski</TopAppBarTitle>
        </Section>
        <Section align="end" toolbar>
          <IconButton aria-label="Search">
            <Icon tag="svg" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiSearchWeb} />
            </Icon>
          </IconButton>
        </Section>
      </Row>
    </TopAppBar>
    {#if view === 'buy'}
      <p>Buy</p>
    {:else if view === 'sell'}
      <p>Sell</p>
    {:else if view == 'login'}
      <p>Login</p>
    {/if}
  </AppContent>
</main>

<style>
  .top-app-bar-container {
    width: 100%;
    height: 500px;
    border: 1px solid
      var(--mdc-theme-text-hint-on-background, rgba(0, 0, 0, 0.1));
    margin: 0 18px 18px 0;
    background-color: var(--mdc-theme-background, #fff);

    overflow: auto;
    display: inline-block;
  }

  @media (max-width: 480px) {
    .top-app-bar-container {
      margin-right: 0;
    }
  }

  .flexy {
    display: flex;
    flex-wrap: wrap;
  }

  .flexor {
    display: inline-flex;
    flex-direction: column;
  }

  .flexor-content {
    flex-basis: 0;
    height: 0;
    flex-grow: 1;
    overflow: auto;
  }
</style>
