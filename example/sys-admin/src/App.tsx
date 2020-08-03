import React from 'react';
import './App.css';

import dataProvider from './DataProvider'
import { Admin, Resource } from 'react-admin';
import { PostList, PostEdit, PostCreate, PostIcon} from './routes/post';

function App() {
  return (
    <Admin dataProvider={dataProvider} >
      <Resource name="post" list={PostList} edit={PostEdit} create={PostCreate} icon={PostIcon} />
    </Admin>
  );
}

export default App;
