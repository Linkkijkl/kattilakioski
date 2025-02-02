<script lang="ts">
    import IconButton, { Icon } from '@smui/icon-button';
    import TopAppBar, { Row, Section, Title as TopAppBarTitle } from '@smui/top-app-bar';
    import { mdiMenu, mdiSearchWeb } from '@mdi/js';
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
  let view = $state("buy");
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
          onclick={() => setView('Inbox')}
          activated={view === 'Inbox'}
        >
          <Graphic class="material-icons" aria-hidden="true">inbox</Graphic>
          <Text>Inbox</Text>
        </Item>
        <Item
          href="javascript:void(0)"
          onclick={() => setView('Star')}
          activated={view === 'Star'}
        >
          <Graphic class="material-icons" aria-hidden="true">star</Graphic>
          <Text>Star</Text>
        </Item>
        <Item
          href="javascript:void(0)"
          onclick={() => setView('Sent Mail')}
          activated={view === 'Sent Mail'}
        >
          <Graphic class="material-icons" aria-hidden="true">send</Graphic>
          <Text>Sent Mail</Text>
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
      <div class="flexor-content">
        <img
          alt="Page content placeholder"
          src="/page-content.jpg"
          style="display: block; max-width: 100%; height: auto; margin: 1em auto;"
        />
      </div>
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
